import sys
from pathlib import Path

DEFAULT_ARRAY_NAME = "WORDS"

def txt_to_rust_array(input_path: str, output_path: str, array_name: str = DEFAULT_ARRAY_NAME):
    input_file = Path(input_path)
    output_file = Path(output_path)

    with input_file.open('r', encoding='utf-8') as f:
        lines = [line.strip() for line in f if line.strip() and not line.startswith("#")]

    with output_file.open('w', encoding='utf-8') as f:
        f.write(f"// Auto-generated from {input_file.name}\n\n")
        f.write(f"pub const LENGTH: usize = {len(lines)};\n")
        f.write(f"pub const {array_name}: [&'static str; LENGTH] = [\n")
        for line in lines:
            escaped = line.replace("\\", "\\\\").replace("\"", "\\\"")
            f.write(f'    "{escaped}",\n')
        f.write("];\n")

if __name__ == "__main__":
    if not 3 <= len(sys.argv) <= 4:
        print("Usage: python txt_to_rust.py input.txt output.rs [ARRAY_NAME]")
        sys.exit(1)
    array_name = (sys.argv[3:4] or [DEFAULT_ARRAY_NAME])[0]
    txt_to_rust_array(sys.argv[1], sys.argv[2], array_name)
