import sys

def get_letters(binary: int):
    start = ord("a")
    result = []
    for i in range(0, 26):
        if binary & 1 << i:
            result.append(chr(start + i))
    return result

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python get_letters.py [number]")
        sys.exit(1)
    print(" ".join(get_letters(int(sys.argv[1]))))