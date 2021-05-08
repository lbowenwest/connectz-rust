import sys

from . import run_file

if len(sys.argv) != 2:
    print(f'{sys.argv[0]}: Provide one input file')
    sys.exit()

result = run_file(sys.argv[1])
print(result)
