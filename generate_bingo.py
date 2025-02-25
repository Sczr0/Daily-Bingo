import random
import json
from datetime import datetime

# 定义颜色和规则
COLORS = ["红", "蓝", "黑", "绿", "黄", "紫", "白"]
RULES = {
    "红": "周围至少勾一个蓝格",
    "蓝": "周围最多勾两个黑格",
    "黑": "必须勾选",
    "绿": "所在行列勾数相同",
    "黄": "所在斜线勾数相同",
    "紫": "周围勾数是奇数",
    "白": "无限制"
}

# 生成5x5网格
def generate_grid():
    grid = []
    for i in range(5):
        row = []
        for j in range(5):
            color = random.choice(COLORS)
            checked = color == "黑"  # 黑格默认勾选
            row.append({"x": i, "y": j, "color": color, "checked": checked})
        grid.append(row)
    return grid

# 生成五连路径
def generate_winning_path(grid):
    paths = [
        "第1行", "第2行", "第3行", "第4行", "第5行",
        "第1列", "第2列", "第3列", "第4列", "第5列",
        "主对角线", "副对角线"
    ]
    return random.choice(paths)

# 保存为JSON文件
def save_to_json(grid, path):
    data = {
        "date": datetime.now().strftime("%Y-%m-%d"),
        "grid": grid,
        "winning_path": path
    }
    with open("bingo.json", "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)

if __name__ == "__main__":
    grid = generate_grid()
    path = generate_winning_path(grid)
    save_to_json(grid, path)
