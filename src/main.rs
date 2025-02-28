use serde::{Serialize, Deserialize};
use image::{ImageBuffer, Rgb};
use imageproc::drawing::{draw_text_mut, draw_line_segment_mut};
use rusttype::{Font, Scale};
use chrono::Local;
use rand::{seq::SliceRandom, Rng};
use log::{info, warn, debug};
use std::{fs, path::Path, time::Instant};
use chrono::{Utc, DateTime};
use chrono_tz::Asia::Shanghai;

// ----------------------------- 数据结构定义 -----------------------------
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Color {
    Red,
    Blue,
    Black,
    Green,
    Yellow,
    Purple,
    White,
    Orange,
    Cyan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Cell {
    x: usize,
    y: usize,
    color: Color,
    checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Grid(Vec<Vec<Cell>>);

// ----------------------------- 规则校验实现 -----------------------------
impl Grid {
    fn get_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        for i in x.saturating_sub(1)..=x.saturating_add(1) {
            for j in y.saturating_sub(1)..=y.saturating_add(1) {
                if i == x && j == y {
                    continue;
                }
                if i < 5 && j < 5 {
                    neighbors.push((i, j));
                }
            }
        }
        neighbors
    }

    fn get_four_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        // 上
        if x > 0 {
            neighbors.push((x - 1, y));
        }
        // 下
        if x < 4 {
            neighbors.push((x + 1, y));
        }
        // 左
        if y > 0 {
            neighbors.push((x, y - 1));
        }
        // 右
        if y < 4 {
            neighbors.push((x, y + 1));
        }
        neighbors
    }

    fn check_red_rule(&self, x: usize, y: usize) -> bool {
        let neighbors = self.get_neighbors(x, y);
        let ok = neighbors.iter().any(|(i, j)| self.0[*i][*j].checked);
        if !ok {
            debug!("❌ 红格({},{})规则不满足", x, y);
        }
        ok
    }

    fn check_blue_rule(&self, x: usize, y: usize) -> bool {
        let neighbors = self.get_neighbors(x, y);
        let ok = neighbors.iter().filter(|(i, j)| self.0[*i][*j].checked).count() <= 2;
        if !ok {
            debug!("❌ 蓝格({},{})规则不满足", x, y);
        }
        ok
    }

    fn check_green_rule(&self, x: usize, y: usize) -> bool {
        let row_count = self.0[x].iter().filter(|cell| cell.checked).count();
        let col_count = (0..5).filter(|i| self.0[*i][y].checked).count();
        let ok = row_count == col_count;
        if !ok {
            debug!("❌ 绿格({},{})规则不满足", x, y);
        }
        ok
    }

    fn check_yellow_rule(&self, x: usize, y: usize) -> bool {
        let diag1 = self.get_diagonal(x, y, (-1, -1), (1, 1));
        let diag2 = self.get_diagonal(x, y, (-1, 1), (1, -1));

        let count1 = diag1.iter().filter(|&&(i, j)| self.0[i][j].checked).count();
        let count2 = diag2.iter().filter(|&&(i, j)| self.0[i][j].checked).count();

        let ok = count1 == count2;
        if !ok {
            debug!("❌ 黄格({},{})规则不满足：对角1勾数={} 对角2勾数={}", x, y, count1, count2);
        }
        ok
    }

    fn get_diagonal(&self, x: usize, y: usize, dir1: (i32, i32), dir2: (i32, i32)) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        let x = x as i32;
        let y = y as i32;

        // 向dir1方向延伸
        let (mut cx, mut cy) = (x as i32, y as i32);
        loop {
            if cx < 0 || cy < 0 || cx >= 5 || cy >= 5 { break; }
            cells.push((cx as usize, cy as usize));
            cx += dir1.0;
            cy += dir1.1;
        }

        // 向dir2方向延伸（跳过中心点）
        let (mut cx, mut cy) = (x as i32, y as i32);
        loop {
            if cx < 0 || cy < 0 || cx >= 5 || cy >= 5 { break; }
            cells.push((cx as usize, cy as usize));
            cx += dir2.0;
            cy += dir2.1;
        }

        cells
    }

    fn check_purple_rule(&self, x: usize, y: usize) -> bool {
        let neighbors = self.get_neighbors(x, y);
        let ok = neighbors.iter().filter(|(i, j)| self.0[*i][*j].checked).count() % 2 == 1;
        if !ok {
            debug!("❌ 紫格({},{})规则不满足", x, y);
        }
        ok
    }

    fn check_orange_rule(&self, x: usize, y: usize) -> bool {
        let neighbors = self.get_neighbors(x, y);
        let count = neighbors.iter().filter(|(i, j)| self.0[*i][*j].checked).count();
        let ok = count % 2 == 0;
        if !ok {
            debug!("❌ 橙格({},{})规则不满足：周围勾选数{}不是偶数", x, y, count);
        }
        ok
    }

    fn check_cyan_rule(&self, x: usize, y: usize) -> bool {
        let cell = &self.0[x][y];
        if !cell.checked {
            return true;
        }
        let neighbors = self.get_four_neighbors(x, y);
        let has_checked = neighbors.iter().any(|(i, j)| self.0[*i][*j].checked);
        if !has_checked {
            debug!("❌ 青格({},{})勾选时周围上下左右无勾选格子", x, y);
        }
        has_checked
    }

    fn check_all_rules(&self) -> bool {
        for i in 0..5 {
            for j in 0..5 {
                let cell = &self.0[i][j];
                let valid = match cell.color {
                    Color::Red => self.check_red_rule(i, j),
                    Color::Blue => self.check_blue_rule(i, j),
                    Color::Green => self.check_green_rule(i, j),
                    Color::Yellow => self.check_yellow_rule(i, j),
                    Color::Purple => self.check_purple_rule(i, j),
                    Color::Orange => self.check_orange_rule(i, j),
                    Color::Cyan => self.check_cyan_rule(i, j),
                    _ => true,
                };
                if !valid {
                    return false;
                }
            }
        }
        true
    }

    fn check_total_checked(&self, max_checked: usize) -> bool {
        let total = self.0.iter().flatten().filter(|cell| cell.checked).count();
        if total > max_checked {
            debug!("❌ 总勾选数超过限制: {} > {}", total, max_checked);
        }
        total <= max_checked
    }

    fn has_five_in_a_row(&self) -> bool {
        // 检查行
        for row in &self.0 {
            for i in 0..=0 {
                if row[i..i+5].iter().all(|cell| cell.checked) {
                    return true;
                }
            }
        }
        // 检查列
        for j in 0..5 {
            for i in 0..=0 {
                if (i..i+5).all(|k| self.0[k][j].checked) {
                    return true;
                }
            }
        }
        // 检查对角线
        for i in 0..=0 {
            for j in 0..=0 {
                if (0..5).all(|k| self.0[i + k][j + k].checked) 
                || (0..5).all(|k| self.0[i + k][4 - j - k].checked) {
                    return true;
                }
            }
        }
        false
    }

    fn new_blank(color_grid: &[Vec<Color>]) -> Self {
        Grid(
            (0..5).map(|i| {
                (0..5).map(|j| Cell {
                    x: i, y: j,
                    color: color_grid[i][j],
                    checked: false,
                }).collect()
            }).collect()
        )
    }
}

// ----------------------------- 求解器实现 -----------------------------
struct Solver {
    color_grid: Vec<Vec<Color>>,
    max_checked: usize,
}

impl Solver {
    fn new(color_grid: Vec<Vec<Color>>, max_checked: usize) -> Self {
        Self { color_grid, max_checked }
    }

    fn initialize_grid(&self) -> Grid {
        Grid(
            (0..5).map(|i| {
                (0..5).map(|j| Cell {
                    x: i,
                    y: j,
                    color: self.color_grid[i][j],
                    checked: self.color_grid[i][j] == Color::Black, // 黑格默认勾选
                }).collect()
            }).collect()
        )
    }

    fn next_position(&self, x: usize, y: usize) -> (usize, usize) {
        if y == 4 { (x + 1, 0) } else { (x, y + 1) }
    }

    fn solve(&self) -> Vec<Grid> {
        let mut solutions = Vec::new();
        let mut current_grid = self.initialize_grid();
        let initial_checked = current_grid.0.iter().flatten().filter(|c| c.checked).count();
        self.backtrack(0, 0, &mut current_grid, &mut solutions, initial_checked);
        solutions
    }

    fn backtrack(&self, x: usize, y: usize, grid: &mut Grid, solutions: &mut Vec<Grid>, current_checked: usize) {
        if x == 5 {
            if grid.check_all_rules() 
                && grid.has_five_in_a_row() 
                && current_checked <= self.max_checked 
            {
                if !solutions.iter().any(|s| s.0 == grid.0) {
                    info!("🎉 找到有效解！总勾选数: {}", current_checked);
                    solutions.push(grid.clone());
                }
            }
            return;
        }
    
        let (next_x, next_y) = self.next_position(x, y);
        
        if self.color_grid[x][y] == Color::Black {
            self.backtrack(next_x, next_y, grid, solutions, current_checked);
        } else {
            // 尝试勾选该单元格
            grid.0[x][y].checked = true;
            let new_checked = current_checked + 1;
            
            // 仅保留总勾选数剪枝
            if new_checked <= self.max_checked {
                self.backtrack(next_x, next_y, grid, solutions, new_checked);
            }
            
            // 回溯，尝试不勾选
            grid.0[x][y].checked = false;
            self.backtrack(next_x, next_y, grid, solutions, current_checked);
        }
    }
}

// ----------------------------- 输出函数 -----------------------------
fn save_solutions_json(solutions: &[Grid], path: &str) {
    let data = serde_json::json!({
        "solutions": solutions,
        "total_solutions": solutions.len(),
    });
    fs::create_dir_all(Path::new(path).parent().unwrap()).unwrap();
    fs::write(path, data.to_string()).unwrap();
}

fn save_grid_image(grid: &Grid, path: &str, show_checks: bool, date: &str, solutions_count: usize) {
    // ----------------------------- 参数配置 -----------------------------
    let cell_size: u32 = 90;        // 单元格尺寸
    let rule_font_size: f32 = 13.5; // 规则文字字号
    let line_spacing: u32 = 22;     // 行间距
    let margin: u32 = 12;           // 全局边距
    let rule_column_width: u32 = 310; // 规则栏宽度

    // ----------------------------- 颜色定义 -----------------------------
    let background_color = Rgb([245u8, 245u8, 245u8]); // 浅灰背景
    let rule_bg_color = Rgb([255u8, 255u8, 255u8]);    // 规则区白色背景
    let text_color = Rgb([80u8, 80u8, 80u8]);          // 深灰文字
    let grid_line_color = Rgb([210u8, 210u8, 210u8]);  // 网格线颜色
    let check_color = Rgb([100u8, 100u8, 100u8]);      // 勾选标记颜色

    // ----------------------------- 布局计算 -----------------------------
    // 规则文本
    let solution_count_str = format!("本日题目共有 {} 个解", solutions_count); // 将 format! 结果存储为局部变量
    let rules = vec![
        " ",
        " ",
        "红格周围至少有一个被勾选的格子。",
        "蓝格周围勾选的格子不得超过两个。",
        "绿格所在行的勾选总数",
        "须等于所在列的勾选总数。",
        "黄格所在两条交叉对角线",
        "（从黄格向四角延伸）的勾选总数必须相等。",
        "紫格周围被勾选的格子数量须为奇数。",
        "橙格周围勾选的格子数量须为偶数。",
        "青格如果被勾选，则其上下左右（不包括对角）",
        "至少有一个被勾选的格子。",
        "黑格必须勾。",
        "每个格子的颜色规则均需满足",
        "最终要把五个勾连起来，加油吧~",
        "-----------------------------------",
        "周围指的是一圈八个格子，不包括自己",
        "五连钩可以是横排竖排，以及两条对角线",
        &solution_count_str, // 使用局部变量的引用
    ];

    // 加载字体
    let font_data: &[u8] = include_bytes!("../fonts/font.ttf");
    let font = Font::try_from_bytes(font_data).unwrap();

    // ----------------------------- 图像尺寸计算 -----------------------------
    // 计算规则文本高度
    let mut text_height = margin;
    let scale = Scale::uniform(rule_font_size);
    for line in &rules {
        let line_count = line.chars().filter(|c| *c == '\n').count() + 1;
        text_height += line_count as u32 * line_spacing;
    }

    // 网格区域参数
    let grid_area_height = 5 * cell_size + margin * 2;
    let footer_height = 30; // 版权信息区域高度
    
    // 总图像尺寸
    let img_width = rule_column_width + 5 * cell_size + margin * 3;
    let img_height = text_height.max(grid_area_height) + footer_height;

    // ----------------------------- 绘制图像 -----------------------------
    let mut img = ImageBuffer::from_pixel(img_width, img_height, background_color);

    // 绘制规则区背景
    for x in 0..rule_column_width {
        for y in 0..img_height {
            img.put_pixel(x, y, rule_bg_color);
        }
    }

    // 绘制规则文本
    let mut y_pos = margin as i32;
    for line in rules {
        draw_text_mut(
            &mut img,
            text_color,
            margin as i32 + 10,
            y_pos,
            scale,
            &font,
            line,
        );
        y_pos += line_spacing as i32 * (line.matches('\n').count() as i32 + 1);
    }

    // 绘制网格区域
    let grid_start_x = rule_column_width + margin;
    let grid_start_y = (img_height - grid_area_height) / 2; // 垂直居中
    for (i, row) in grid.0.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            // 单元格颜色
            let color = match cell.color {
                Color::Red => [255, 50, 50],
                Color::Blue => [70, 130, 180],
                Color::Black => [40, 40, 40],
                Color::Green => [50, 205, 50],
                Color::Yellow => [255, 215, 0],
                Color::Purple => [128, 0, 128],
                Color::White => [255, 255, 255],
                Color::Orange => [255, 165, 0],
                Color::Cyan => [0, 255, 255],
            };

            // 单元格坐标
            let x = grid_start_x + j as u32 * cell_size;
            let y = grid_start_y + i as u32 * cell_size;

            // 绘制单元格背景
            for dx in 0..cell_size {
                for dy in 0..cell_size {
                    img.put_pixel(x + dx, y + dy, Rgb(color));
                }
            }

            // 绘制单元格边框
            for dx in 0..cell_size {
                img.put_pixel(x + dx, y, grid_line_color); // 上边框
                img.put_pixel(x + dx, y + cell_size - 1, grid_line_color); // 下边框
            }
            for dy in 0..cell_size {
                img.put_pixel(x, y + dy, grid_line_color); // 左边框
                img.put_pixel(x + cell_size - 1, y + dy, grid_line_color); // 右边框
            }

            // 绘制勾选标记
            if show_checks && cell.checked {
                draw_line_segment_mut(
                    &mut img,
                    (x as f32 + 10.0, y as f32 + 10.0),
                    (x as f32 + cell_size as f32 - 10.0, y as f32 + cell_size as f32 - 10.0),
                    check_color,
                );
                draw_line_segment_mut(
                    &mut img,
                    (x as f32 + 10.0, y as f32 + cell_size as f32 - 10.0),
                    (x as f32 + cell_size as f32 - 10.0, y as f32 + 10.0),
                    check_color,
                );
            }
        }
    }

    // ----------------------------- 版权信息 -----------------------------
    let footer = format!("Generated by BingoSolver @ {}", date);
    let footer_scale = Scale::uniform(12.0);
    draw_text_mut(
        &mut img,
        text_color,
        margin as i32 + 10, // 与规则文字左对齐
        (img_height - footer_height + 8) as i32, // 保持在同一高度
        footer_scale,
        &font,
        &footer
    );

    img.save(path).unwrap();
}

fn move_to_date_folder(date: &str) {
    let date_folder = format!("data/{}", date);
    if Path::new(&date_folder).exists() {
        fs::remove_dir_all(&date_folder).unwrap();
    }
    fs::create_dir_all(&date_folder).unwrap();

    // 需要保留在根目录的文件名
    let keep_files = vec!["solutions.json", "blank.png"];

    for entry in fs::read_dir("data").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // 仅处理文件，跳过目录
        if !path.is_file() {
            continue;
        }

        let file_name = entry.file_name();
        let file_name_str = file_name.to_str().unwrap();

        // 仅移动非保留文件（且不是当前日期的文件夹）
        if !keep_files.contains(&file_name_str) && file_name_str != date {
            let new_path = format!("{}/{}", date_folder, file_name_str);
            fs::rename(&path, new_path).unwrap();
        }
    }
}

// ----------------------------- 主函数 -----------------------------
fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp_millis()
        .format_module_path(false)
        .filter_level(log::LevelFilter::Info)
        .init();
    info!("程序启动");

    fs::create_dir_all("data").expect("无法创建data目录");

    let (solutions, date, color_grid) = loop {
        let utc_time = Utc::now();
        let beijing_time: DateTime<chrono_tz::Tz> = utc_time.with_timezone(&Shanghai);
        let date = beijing_time.format("%Y-%m-%d").to_string();
        
        // 生成新的颜色网格
        let color_grid = generate_color_grid();
        info!("生成新题目布局:\n{}", format_grid_colors(&color_grid));

        let solver = Solver::new(color_grid.clone(), 25);
        let solutions = solver.solve();
        
        if !solutions.is_empty() {
            break (solutions, date, color_grid);
        }
        warn!("未找到解，重新生成题目...");
    };

    // 保存到根目录
    save_solutions_json(&solutions, "data/solutions.json");
    save_grid_image(
        &Grid::new_blank(&color_grid), 
        "data/blank.png", 
        false, 
        &date,
        solutions.len() // 传递解数量
    );

    // 保存到日期文件夹
    move_to_date_folder(&date);
    save_solutions_json(&solutions, &format!("data/{}/solutions.json", date));
    for (i, solution) in solutions.iter().enumerate() {
        save_grid_image(
            solution, 
            &format!("data/{}/solution_{}.png", date, i), 
            true, 
            &date,
            solutions.len() // 传递解数量
        );
    }
    save_grid_image(
        &Grid::new_blank(&color_grid), 
        &format!("data/{}/blank.png", date), 
        false, 
        &date,
        solutions.len()
    );

    info!("结果已保存至 data/ 和 data/{}/ 文件夹", date);
}

// ----------------------------- 工具函数 -----------------------------
fn generate_color_grid() -> Vec<Vec<Color>> {
    let mut rng = rand::thread_rng();
    let colors = vec![
        Color::Red, Color::Blue, Color::Black,
        Color::Green, Color::Yellow, Color::Purple, 
        Color::White, Color::Orange, Color::Cyan,
    ];
    
    // 生成初始随机网格
    let mut grid: Vec<Vec<Color>> = (0..5)
        .map(|_| (0..5).map(|_| *colors.choose(&mut rng).unwrap()).collect())
        .collect();

    // 强制至少有10个白格
    let mut white_count = grid.iter().flatten().filter(|c| **c == Color::White).count();
    while white_count < 10 {
        let x = rng.gen_range(0..5);
        let y = rng.gen_range(0..5);
        if grid[x][y] != Color::White {
            grid[x][y] = Color::White;
            white_count += 1;
        }
    }

    grid
}

fn format_grid_colors(grid: &[Vec<Color>]) -> String {
    grid.iter().map(|row| {
        row.iter().map(|color| match color {
            Color::Red => "红",
            Color::Blue => "蓝",
            Color::Black => "黑",
            Color::Green => "绿",
            Color::Yellow => "黄",
            Color::Purple => "紫",
            Color::White => "白",
            Color::Orange => "橙",
            Color::Cyan => "青",
        }).collect::<Vec<_>>().join(" ")
    }).collect::<Vec<_>>().join("\n")
}