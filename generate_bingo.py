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

# 全局变量：勾数限制
MAX_CHECKED = 15  # 可以在脚本中更改

# 获取周围格子的坐标（包括八邻域）
def get_neighbors(x, y):
    neighbors = []
    for i in range(x - 1, x + 2):
        for j in range(y - 1, y + 2):
            if i == x and j == y:
                continue  # 排除自己
            if 0 <= i < 5 and 0 <= j < 5:
                neighbors.append((i, j))
    return neighbors

# 检查红格规则：周围至少勾选一个蓝格
def check_red_rule(grid, x, y):
    neighbors = get_neighbors(x, y)
    for i, j in neighbors:
        if grid[i][j]["color"] == "蓝" and grid[i][j]["checked"]:
            return True
    return False

# 检查蓝格规则：周围最多勾选两个黑格
def check_blue_rule(grid, x, y):
    neighbors = get_neighbors(x, y)
    black_count = 0
    for i, j in neighbors:
        if grid[i][j]["color"] == "黑" and grid[i][j]["checked"]:
            black_count += 1
    return black_count <= 2

# 检查绿格规则：所在行列勾选数相同
def check_green_rule(grid, x, y):
    row_count = sum(1 for cell in grid[x] if cell["checked"])
    col_count = sum(1 for i in range(5) if grid[i][y]["checked"])
    return row_count == col_count

# 检查黄格规则：所在斜线勾选数相同
def check_yellow_rule(grid, x, y):
    diag1_count = sum(1 for i in range(5) if grid[i][i]["checked"])
    diag2_count = sum(1 for i in range(5) if grid[i][4 - i]["checked"])
    return diag1_count == diag2_count

# 检查紫格规则：周围勾选数为奇数
def check_purple_rule(grid, x, y):
    neighbors = get_neighbors(x, y)
    checked_count = sum(1 for i, j in neighbors if grid[i][j]["checked"])
    return checked_count % 2 == 1

# 检查所有规则
def check_all_rules(grid):
    for i in range(5):
        for j in range(5):
            cell = grid[i][j]
            color = cell["color"]
            if color == "红" and not check_red_rule(grid, i, j):
                return False
            if color == "蓝" and not check_blue_rule(grid, i, j):
                return False
            if color == "绿" and not check_green_rule(grid, i, j):
                return False
            if color == "黄" and not check_yellow_rule(grid, i, j):
                return False
            if color == "紫" and not check_purple_rule(grid, i, j):
                return False
    return True

# 检查总勾数是否在限制内
def check_total_checked(grid):
    total_checked = sum(1 for row in grid for cell in row if cell["checked"])
    return total_checked <= MAX_CHECKED

# 生成5x5网格，并确保符合所有规则
def generate_grid():
    global MAX_CHECKED  # 在函数内部声明为全局变量
    while True:
        grid = []
        for i in range(5):
            row = []
            for j in range(5):
                color = random.choice(COLORS)
                # 黑格默认勾选，其他格随机勾选
                checked = color == "黑" or random.choice([True, False])
                row.append({"x": i, "y": j, "color": color, "checked": checked})
            grid.append(row)
        
        # 检查是否至少有一个五连钩、符合所有规则且总勾数在限制内
        if has_five_in_a_row(grid) and check_all_rules(grid) and check_total_checked(grid):
            break
    
    return grid

# 检查是否至少有一个五连钩
def has_five_in_a_row(grid):
    # 检查行
    for row in grid:
        if all(cell["checked"] for cell in row):
            return True
    
    # 检查列
    for j in range(5):
        if all(grid[i][j]["checked"] for i in range(5)):
            return True
    
    # 检查主对角线
    if all(grid[i][i]["checked"] for i in range(5)):
        return True
    
    # 检查副对角线
    if all(grid[i][4 - i]["checked"] for i in range(5)):
        return True
    
    return False

# 生成宾果题目图片（仅颜色块）
def generate_bingo_image(grid, date_str):
    # 创建空白图片（400x400像素）
    img = Image.new("RGB", (400, 400), "white")
    draw = ImageDraw.Draw(img)

    # 定义颜色映射
    colors = {
        "红": "#FF0000", "蓝": "#0000FF", "黑": "#000000",
        "绿": "#00FF00", "黄": "#FFFF00", "紫": "#800080", "白": "#FFFFFF"
    }

    # 绘制5x5网格
    cell_size = 80
    for i in range(5):
        for j in range(5):
            x = j * cell_size
            y = i * cell_size
            cell = grid[i][j]
            
            # 绘制格子背景色
            draw.rectangle([x, y, x+cell_size, y+cell_size], fill=colors[cell["color"]])

    # 保存图片
    os.makedirs("images", exist_ok=True)
    image_path = f"images/bingo_{date_str}.png"
    img.save(image_path)
    return image_path

# 保存为JSON文件
def save_to_json(grid, date_str):
    data = {
        "date": date_str,
        "grid": grid,
        "total_checked": sum(1 for row in grid for cell in row if cell["checked"])
    }
    os.makedirs("data", exist_ok=True)
    json_path = f"data/bingo_{date_str}.json"
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    return json_path

if __name__ == "__main__":
    # 可以在脚本中更改勾数限制
    global MAX_CHECKED
    MAX_CHECKED = 15  # 默认限制为15

    # 生成网格
    grid = generate_grid()
    
    # 获取当前日期
    date_str = datetime.now().strftime("%Y-%m-%d")
    
    # 生成图片
    image_path = generate_bingo_image(grid, date_str)
    print(f"图片已生成: {image_path}")
    
    # 保存为JSON文件
    json_path = save_to_json(grid, date_str)
    print(f"JSON已生成: {json_path}")
