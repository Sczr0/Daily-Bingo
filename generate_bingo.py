import random
import json
from datetime import datetime
from PIL import Image, ImageDraw
import os

# 定义颜色
COLORS = ["红", "蓝", "黑", "绿", "黄", "紫", "白"]

# 定义规则
RULES = {
    "红": "周围至少勾选一个蓝格",
    "蓝": "周围最多勾选两个黑格",
    "黑": "必须勾选",
    "绿": "所在行列勾选数相同",
    "黄": "所在斜线勾选数相同",
    "紫": "周围勾选数为奇数",
    "白": "无限制"
}

# 全局变量配置
GLOBAL_CONFIG = {
    "MAX_CHECKED": 15,       # 勾数限制
    "MAX_SOLUTIONS": 1000,   # 最大解数量（仅限所有解模式）
    "GENERATE_ALL": True    # 是否生成所有解（True为所有解模式，False为单解模式）
}

# ----------------------------- 通用工具函数 -----------------------------
def get_neighbors(x, y):
    """获取周围八邻域坐标"""
    neighbors = []
    for i in range(x - 1, x + 2):
        for j in range(y - 1, y + 2):
            if i == x and j == y:
                continue  # 排除自己
            if 0 <= i < 5 and 0 <= j < 5:
                neighbors.append((i, j))
    return neighbors

def has_five_in_a_row(grid):
    """检查是否至少有一个五连钩"""
    # 检查行
    for row in grid:
        if all(cell["checked"] for cell in row):
            return True
    # 检查列
    for j in range(5):
        if all(grid[i][j]["checked"] for i in range(5)):
            return True
    # 检查对角线
    if all(grid[i][i]["checked"] for i in range(5)) or all(grid[i][4 - i]["checked"] for i in range(5)):
        return True
    return False

# ----------------------------- 规则校验函数 -----------------------------
def check_red_rule(grid, x, y):
    neighbors = get_neighbors(x, y)
    return any(grid[i][j]["color"] == "蓝" and grid[i][j]["checked"] for i, j in neighbors)

def check_blue_rule(grid, x, y):
    neighbors = get_neighbors(x, y)
    black_checked = sum(1 for i, j in neighbors if grid[i][j]["color"] == "黑" and grid[i][j]["checked"])
    return black_checked <= 2

def check_green_rule(grid, x, y):
    row_count = sum(1 for cell in grid[x] if cell["checked"])
    col_count = sum(1 for i in range(5) if grid[i][y]["checked"])
    return row_count == col_count

def check_yellow_rule(grid, x, y):
    diag1 = sum(1 for i in range(5) if grid[i][i]["checked"])
    diag2 = sum(1 for i in range(5) if grid[i][4 - i]["checked"])
    return diag1 == diag2

def check_purple_rule(grid, x, y):
    neighbors = get_neighbors(x, y)
    checked_count = sum(1 for i, j in neighbors if grid[i][j]["checked"])
    return checked_count % 2 == 1

def check_all_rules(grid):
    """综合校验所有规则"""
    for i in range(5):
        for j in range(5):
            color = grid[i][j]["color"]
            check_func = {
                "红": check_red_rule,
                "蓝": check_blue_rule,
                "绿": check_green_rule,
                "黄": check_yellow_rule,
                "紫": check_purple_rule
            }.get(color, lambda *_: True)  # 白格无规则
            if not check_func(grid, i, j):
                return False
    return True

def check_total_checked(grid):
    """勾数限制校验"""
    total = sum(1 for row in grid for cell in row if cell["checked"])
    return total <= GLOBAL_CONFIG["MAX_CHECKED"]

# ----------------------------- 生成模式逻辑 -----------------------------
def generate_single_grid():
    """生成单解模式：随机生成直到符合条件"""
    while True:
        grid = []
        for i in range(5):
            row = []
            for j in range(5):
                color = random.choice(COLORS)
                checked = color == "黑" or random.choice([True, False])
                row.append({"x": i, "y": j, "color": color, "checked": checked})
            grid.append(row)
        if has_five_in_a_row(grid) and check_all_rules(grid) and check_total_checked(grid):
            return grid

def generate_all_solutions():
    """所有解模式：回溯遍历所有可能性"""
    # 生成固定颜色网格
    color_grid = [[random.choice(COLORS) for _ in range(5)] for _ in range(5)]
    solutions = []
    
    def backtrack(x, y, current_grid):
        if x == 5:
            if has_five_in_a_row(current_grid) and check_all_rules(current_grid) and check_total_checked(current_grid):
                solutions.append([row.copy() for row in current_grid])
            return
        next_x = x + (y + 1) // 5
        next_y = (y + 1) % 5
        
        # 黑格必须勾选
        if color_grid[x][y] == "黑":
            current_grid[x][y]["checked"] = True
            backtrack(next_x, next_y, current_grid)
        else:
            # 尝试勾选
            current_grid[x][y]["checked"] = True
            if check_all_rules(current_grid):  # 提前剪枝
                backtrack(next_x, next_y, current_grid)
            # 尝试不勾选
            current_grid[x][y]["checked"] = False
            if check_all_rules(current_grid):  # 提前剪枝
                backtrack(next_x, next_y, current_grid)
    
    # 初始化网格结构
    initial_grid = [
        [{"x": i, "y": j, "color": color_grid[i][j], "checked": False} 
        for j in range(5)
    ] for i in range(5)]
    
    backtrack(0, 0, initial_grid)
    return solutions[:GLOBAL_CONFIG["MAX_SOLUTIONS"]]  # 限制最大解数量

# ----------------------------- 输出函数 -----------------------------
def generate_bingo_image(grid, date_str):
    """生成图片"""
    img = Image.new("RGB", (400, 400), "white")
    draw = ImageDraw.Draw(img)
    colors = {
        "红": "#FF0000", "蓝": "#0000FF", "黑": "#000000",
        "绿": "#00FF00", "黄": "#FFFF00", "紫": "#800080", "白": "#FFFFFF"
    }
    cell_size = 80
    for i in range(5):
        for j in range(5):
            x, y = j * cell_size, i * cell_size
            draw.rectangle([x, y, x+cell_size, y+cell_size], fill=colors[grid[i][j]["color"]])
    os.makedirs("images", exist_ok=True)
    image_path = f"images/bingo_{date_str}.png"
    img.save(image_path)
    return image_path

def save_solutions(solutions, date_str, mode):
    """保存解到JSON"""
    os.makedirs("data", exist_ok=True)
    if mode == "single":
        data = {
            "date": date_str,
            "grid": solutions[0],
            "total_checked": sum(1 for row in solutions[0] for cell in row if cell["checked"])
        }
        json_path = f"data/bingo_single_{date_str}.json"
    else:
        data = {
            "date": date_str,
            "solutions": solutions,
            "total_solutions": len(solutions)
        }
        json_path = f"data/bingo_all_{date_str}.json"
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    return json_path

# ----------------------------- 主函数 -----------------------------
if __name__ == "__main__":
    date_str = datetime.now().strftime("%Y-%m-%d")
    
    if GLOBAL_CONFIG["GENERATE_ALL"]:
        # 所有解模式
        solutions = generate_all_solutions()
        json_path = save_solutions(solutions, date_str, "all")
        print(f"生成完成！共找到 {len(solutions)} 个解，保存至 {json_path}")
    else:
        # 单解模式
        grid = generate_single_grid()
        image_path = generate_bingo_image(grid, date_str)
        json_path = save_solutions([grid], date_str, "single")
        print(f"图片已生成: {image_path}")
        print(f"JSON已生成: {json_path}")
