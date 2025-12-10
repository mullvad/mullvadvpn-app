#!/usr/bin/env python3
"""
Visualize the number of open issues over time from local JSON files.
Generates an interactive HTML page with label filtering.
"""

import json
import os
import sys
from datetime import datetime
from collections import defaultdict
from pathlib import Path


def load_issues_from_directory(directory):
    """Load all issues from JSON files in the directory."""
    issues = []
    json_files = list(Path(directory).glob("*.json"))

    print(f"Loading issues from {directory}...")
    for json_file in json_files:
        try:
            with open(json_file, 'r') as f:
                issue = json.load(f)
                # Filter out pull requests
                if 'pull_request' not in issue:
                    issues.append(issue)
        except Exception as e:
            print(f"  Warning: Failed to load {json_file}: {e}")

    print(f"Loaded {len(issues)} issues (excluding pull requests)")
    return issues


def extract_labels(issues):
    """Extract all unique labels from issues."""
    labels = set()
    for issue in issues:
        for label in issue.get('labels', []):
            labels.add(label['name'])

    return sorted(labels)


def calculate_timeline_for_filter(issues, label_filter=None):
    """Calculate the number of open issues over time, optionally filtered by label."""
    # Filter issues by label if specified
    if label_filter:
        filtered_issues = [
            issue for issue in issues
            if any(label['name'] == label_filter for label in issue.get('labels', []))
        ]
    else:
        filtered_issues = issues

    events = []

    # Create events for issue creation and closure
    for issue in filtered_issues:
        created_at = datetime.fromisoformat(issue['created_at'].replace('Z', '+00:00'))
        events.append((created_at, 1))  # +1 for opening

        if issue['closed_at']:
            closed_at = datetime.fromisoformat(issue['closed_at'].replace('Z', '+00:00'))
            events.append((closed_at, -1))  # -1 for closing

    # Sort events by time
    events.sort(key=lambda x: x[0])

    # Calculate running total
    timeline = []
    open_count = 0

    for timestamp, delta in events:
        open_count += delta
        timeline.append({
            'date': timestamp.strftime('%Y-%m-%d %H:%M:%S'),
            'count': open_count
        })

    return timeline


def generate_html(issues, labels, repo_name="mullvad/mullvadvpn-app"):
    """Generate an HTML page with interactive label filtering."""

    # Calculate timelines for all labels
    print("Calculating timelines...")
    timelines = {}

    # All issues
    print("  All issues...")
    timelines['All issues'] = calculate_timeline_for_filter(issues, None)

    # Each label
    for label in labels:
        print(f"  {label}...")
        timelines[label] = calculate_timeline_for_filter(issues, label)

    # Create fixed color mapping for each label
    colors = [
        '#2196F3', '#4CAF50', '#FF9800', '#E91E63', '#9C27B0',
        '#00BCD4', '#FF5722', '#795548', '#607D8B', '#FFC107',
        '#8BC34A', '#3F51B5', '#F44336', '#009688', '#CDDC39',
        '#FF6F00', '#673AB7', '#00897B', '#C62828', '#5E35B1',
        '#D81B60', '#00ACC1', '#6D4C41', '#1565C0', '#EF6C00'
    ]
    color_map = {'All issues': colors[0]}
    for i, label in enumerate(labels):
        color_map[label] = colors[(i + 1) % len(colors)]

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Open Issues Over Time - {repo_name}</title>
    <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-bottom: 10px;
        }}
        .subtitle {{
            color: #666;
            margin-bottom: 20px;
        }}
        .controls {{
            margin-bottom: 20px;
            padding: 20px;
            background-color: #f9f9f9;
            border-radius: 8px;
        }}
        .controls h3 {{
            margin-top: 0;
            margin-bottom: 15px;
            color: #333;
            font-size: 16px;
        }}
        .checkbox-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
            gap: 12px;
        }}
        .checkbox-item {{
            display: flex;
            align-items: center;
        }}
        .checkbox-item input[type="checkbox"] {{
            width: 18px;
            height: 18px;
            margin-right: 8px;
            cursor: pointer;
        }}
        .checkbox-item label {{
            cursor: pointer;
            font-weight: normal;
            color: #999;
            user-select: none;
            transition: color 0.2s;
        }}
        .checkbox-item input[type="checkbox"]:checked + label {{
            font-weight: 600;
        }}
        .checkbox-item.all-issues {{
            grid-column: 1 / -1;
            padding-bottom: 10px;
            border-bottom: 1px solid #ddd;
            margin-bottom: 5px;
        }}
        .checkbox-item.all-issues label {{
            font-weight: 600;
        }}
        #chart {{
            width: 100%;
            height: 600px;
        }}
        .stats {{
            margin-top: 20px;
            padding: 15px;
            background-color: #f9f9f9;
            border-radius: 4px;
            color: #666;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Open Issues Over Time</h1>
        <div class="subtitle">Repository: <a href="https://github.com/{repo_name}">{repo_name}</a></div>

        <div class="controls">
            <h3>Filter by labels:</h3>
            <div class="checkbox-grid">
                <div class="checkbox-item all-issues">
                    <input type="checkbox" id="filter-all-issues" value="All issues" checked data-color="{color_map['All issues']}">
                    <label for="filter-all-issues">All issues</label>
                </div>
                {''.join(f'''<div class="checkbox-item">
                    <input type="checkbox" id="filter-{label.replace(" ", "-").lower()}" value="{label}" data-color="{color_map[label]}">
                    <label for="filter-{label.replace(" ", "-").lower()}">{label}</label>
                </div>''' for label in labels)}
            </div>
        </div>

        <div id="chart"></div>
        <div class="stats" id="stats"></div>
    </div>

    <script>
        const timelines = {json.dumps(timelines)};
        const colorMap = {json.dumps(color_map)};

        function updateChart() {{
            // Get all checked checkboxes
            const checkboxes = document.querySelectorAll('.checkbox-grid input[type="checkbox"]:checked');
            const selectedFilters = Array.from(checkboxes).map(cb => cb.value);

            if (selectedFilters.length === 0) {{
                document.getElementById('stats').innerHTML = 'Please select at least one filter.';
                Plotly.newPlot('chart', [], {{}});
                return;
            }}

            // Create traces for each selected filter
            const traces = selectedFilters.map(filter => {{
                const data = timelines[filter];

                if (!data || data.length === 0) {{
                    return null;
                }}

                const dates = data.map(d => d.date);
                const counts = data.map(d => d.count);

                return {{
                    x: dates,
                    y: counts,
                    type: 'scatter',
                    mode: 'lines',
                    name: filter,
                    line: {{
                        color: colorMap[filter],
                        width: 2
                    }}
                }};
            }}).filter(trace => trace !== null);

            const layout = {{
                title: '',
                xaxis: {{
                    title: 'Date',
                    showgrid: true,
                    gridcolor: '#e0e0e0'
                }},
                yaxis: {{
                    title: 'Number of Open Issues',
                    showgrid: true,
                    gridcolor: '#e0e0e0'
                }},
                hovermode: 'closest',
                legend: {{
                    x: 0,
                    xanchor: 'left',
                    y: 1
                }},
                margin: {{
                    l: 60,
                    r: 30,
                    t: 30,
                    b: 60
                }}
            }};

            const config = {{
                responsive: true,
                displayModeBar: true,
                displaylogo: false
            }};

            Plotly.newPlot('chart', traces, layout, config);

            // Update stats
            let statsHtml = '';
            selectedFilters.forEach(filter => {{
                const data = timelines[filter];
                if (data && data.length > 0) {{
                    const counts = data.map(d => d.count);
                    const currentCount = counts[counts.length - 1];
                    statsHtml += `<strong>${{filter}}:</strong> ${{currentCount}} open | `;
                }}
            }});

            document.getElementById('stats').innerHTML = statsHtml.slice(0, -3); // Remove trailing ' | '
        }}

        // Function to update label colors based on checkbox state
        function updateLabelColors() {{
            document.querySelectorAll('.checkbox-grid input[type="checkbox"]').forEach(checkbox => {{
                const label = checkbox.nextElementSibling;
                if (checkbox.checked) {{
                    label.style.color = checkbox.getAttribute('data-color');
                }} else {{
                    label.style.color = '#999';
                }}
            }});
        }}

        // Initial setup
        updateLabelColors();
        updateChart();

        // Listen for checkbox changes
        document.querySelectorAll('.checkbox-grid input[type="checkbox"]').forEach(checkbox => {{
            checkbox.addEventListener('change', function() {{
                updateLabelColors();
                updateChart();
            }});
        }});
    </script>
</body>
</html>"""

    return html


def main():
    if len(sys.argv) < 2:
        print("Usage: visualize_issues_timeline.py <issues_directory> [output_file]")
        print("Example: visualize_issues_timeline.py mullvadvpn-app.issues/")
        sys.exit(1)

    issues_dir = sys.argv[1]
    output_file = sys.argv[2] if len(sys.argv) > 2 else "issues_timeline.html"

    if not os.path.isdir(issues_dir):
        print(f"Error: {issues_dir} is not a directory")
        sys.exit(1)

    try:
        # Load issues
        issues = load_issues_from_directory(issues_dir)

        if not issues:
            print("Error: No issues found")
            sys.exit(1)

        # Extract labels
        print("Extracting labels...")
        labels = extract_labels(issues)
        print(f"Found {len(labels)} unique labels: {', '.join(labels)}")

        # Generate HTML
        print("Generating HTML...")
        html = generate_html(issues, labels)

        # Write to file
        with open(output_file, 'w') as f:
            f.write(html)

        print(f"\nSuccess! Open {output_file} in your browser to view the visualization.")

    except Exception as e:
        print(f"Error: {e}")
        raise


if __name__ == "__main__":
    main()
