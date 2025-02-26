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
    let cell_size: u32 = 80;
    let line_height: u32 = 25;
    let rules = vec![
        "红格周围至少有一个被勾选的格子。",
        "蓝格周围被勾选的格子数量不得超过两个。",
        "黑格必须勾。",
        "绿格所在行的勾选总数必须等于所在列的勾选总数。",
        "黄格所在两条交叉对角线\n（从黄格向四角延伸）的勾选总数必须相等。",
        "紫格周围被勾选的格子数量必须为奇数。",
        "每个格子的颜色规则均需满足，\n五个勾连起来证明你不是智障",
        &format!("{}", date),
        &format!("本日题目共有 {} 个解", solutions_count)
    ].join("\n");

    let text_line_count = rules.lines().count() as u32;
    let text_height = text_line_count * line_height;
    let grid_start_y = 10 + text_height + 10;
    let img_height = grid_start_y + 5 * cell_size + 30;
    
    let mut img = ImageBuffer::from_pixel(400, img_height, Rgb([255u8, 255, 255]));

    let font_data: &[u8] = include_bytes!("../fonts/font.ttf");
    let font = Font::try_from_bytes(font_data).unwrap();
    let scale = Scale::uniform(15.0);

    let text_color = Rgb([0u8, 0, 0]);
    let mut y_pos = 10;
    for line in rules.lines() {
        draw_text_mut(
            &mut img, text_color, 10, y_pos as i32, scale, &font, line
        );
        y_pos += line_height as i32;
    }

    for (i, row) in grid.0.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            let color = match cell.color {
                Color::Red => [255u8, 0, 0],
                Color::Blue => [0u8, 0, 255],
                Color::Black => [0u8, 0, 0],
                Color::Green => [0u8, 255, 0],
                Color::Yellow => [255u8, 255, 0],
                Color::Purple => [128u8, 0, 128],
                Color::White => [255u8, 255, 255],
            };
            
            for dx in 0..cell_size {
                for dy in 0..cell_size {
                    img.put_pixel(
                        (j as u32) * cell_size + dx,
                        grid_start_y + (i as u32) * cell_size + dy,
                        Rgb(color),
                    );
                }
            }

            if show_checks && cell.checked {
                let x_start = (j as u32) * cell_size;
                let y_start = grid_start_y + (i as u32) * cell_size;
                // 定义线条颜色为灰色
                let line_color = Rgb([169u8, 169, 169]);
                // 绘制"x"号，通过增加偏移量来模拟加粗效果
                for offset in -2..=2 { // 增加偏移量范围以达到加粗效果
                    draw_line_segment_mut(
                        &mut img,
                        (
                            x_start as f32 + 5.0 + offset as f32, 
                            y_start as f32 + 5.0
                        ),
                        (
                            (x_start + cell_size - 5) as f32 + offset as f32, 
                            (y_start + cell_size - 5) as f32
                        ),
                        line_color,
                    );
                    draw_line_segment_mut(
                        &mut img,
                        (
                            x_start as f32 + 5.0 + offset as f32, 
                            (y_start + cell_size - 5) as f32
                        ),
                        (
                            (x_start + cell_size - 5) as f32 + offset as f32, 
                            y_start as f32 + 5.0
                        ),
                        line_color,
                    );
                }
            }
        }
    }
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

        let solver = Solver::new(color_grid.clone(), 15);
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
        Color::Green, Color::Yellow, Color::Purple, Color::White,
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
        }).collect::<Vec<_>>().join(" ")
    }).collect::<Vec<_>>().join("\n")
}
