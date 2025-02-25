# 每日宾果题目

| 坐标  | 颜色 | 是否勾选 |
|-------|------|----------|
{% for row in grid %}
{% for cell in row %}
| ({{ cell.x }},{{ cell.y }}) | {{ cell.color }} | {{ "是" if cell.checked else "否" }} |
{% endfor %}
{% endfor %}

**五连路径**: {{ winning_path }}
