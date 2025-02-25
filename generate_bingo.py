import random
import json
from datetime import datetime
from PIL import Image, ImageDraw
import os

# 定义颜色
COLORS = ["红", "蓝", "黑", "绿", "黄", "紫", "白"]

# 生成5x5网格
def generate_grid():
    grid = []
    for i in range(5):
        row = []
        for j in range(5):
            color = random.choice(COLORS)
            row.append({"x": i, "y": j, "color": color})
        grid.append(row)
    return grid

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
        "grid": grid
    }
    os.makedirs("data", exist_ok=True)
    json_path = f"data/bingo_{date_str}.json"
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    return json_path

if __name__ == "__main__":
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
