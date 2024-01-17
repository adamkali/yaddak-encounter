import json
import re
import snakecase

def clean_html_tags(text):
    clean = re.compile('<.*?>')
    return re.sub(clean, '', text)

def clean_json(json_data):
    
    try:
        json_data["Traits"] = clean_html_tags(json_data["Traits"])
    except KeyError:
        json_data["Traits"] = None

    try:
        json_data["Actions"] = clean_html_tags(json_data["Actions"])
    except KeyError:
        json_data["Actions"] = None

    try:
        json_data["Legendary Actions"] = clean_html_tags(json_data["Legendary Actions"])
    except KeyError:
        json_data["Legendary Actions"] = None

    # Convert keys to snake case
    snake_case_data = {snakecase.convert(key): value for key, value in json_data.items()}

    return snake_case_data

def process_json_file(input_file, output_file):
    with open(input_file, 'r') as f:
        original_data = json.load(f)

    # Clean the HTML tags and convert keys to snake case

    cleaned_data = []
    for i in original_data:
        datum = clean_json(i)
        cleaned_data.append(datum)

    with open(output_file, 'w') as f:
        # Serialize the cleaned data to a JSON string and write to the output file
        json.dump(cleaned_data, f, indent=4)

# Replace 'input.json' and 'output.json' with your actual file names
process_json_file('./monsters.json', '../monsters.json')

