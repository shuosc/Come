import subprocess
import difflib
import sys
from os import listdir, remove
CASE_DIR = "./integration-test/cases/"

try:
    from colorama import Fore, Back, Style, init
    init()
except ImportError:  # fallback so that the imported classes always exist
    class ColorFallback():
        def __getattr__(self, name): return ''
    Fore = Back = Style = ColorFallback()

def color_diff(diff):
    for line in diff:
        if line.startswith('+'):
            yield Fore.GREEN + line + Fore.RESET
        elif line.startswith('-'):
            yield Fore.RED + line + Fore.RESET
        elif line.startswith('^'):
            yield Fore.BLUE + line + Fore.RESET
        else:
            yield line

for casename in listdir(CASE_DIR):
    print("run case: " + casename)
    subprocess.run(["./target/debug/come",
                    "-i", "{}{}/{}.come".format(CASE_DIR, casename, casename),
                    "-o", "{}{}/{}.asm".format(CASE_DIR, casename, casename),
                    "--emit-ir", "{}{}/{}.cmir".format(CASE_DIR, casename, casename)])
    ir = open("{}{}/{}.cmir".format(CASE_DIR, casename, casename), "r").read()
    correct_ir = open("{}{}/expected/{}.cmir".format(CASE_DIR,
                      casename, casename), "r").read()
    ir_diff = list(difflib.unified_diff(ir, correct_ir, fromfile='result ir', tofile='correct ir'))
    if len(ir_diff) != 0:
        sys.stdout.writelines(color_diff(ir_diff))
        exit(1)
    asm = open("{}{}/{}.asm".format(CASE_DIR, casename, casename), "r").read()
    correct_asm = open("{}{}/expected/{}.asm".format(CASE_DIR,
                       casename, casename), "r").read()
    asm_diff = list(difflib.unified_diff(
        asm, correct_asm, fromfile='result asm', tofile='correct asm'))
    if len(asm_diff) != 0:
        remove("{}{}/{}.cmir".format(CASE_DIR, casename, casename))
        sys.stdout.writelines(color_diff(asm_diff))
        exit(1)
    remove("{}{}/{}.cmir".format(CASE_DIR, casename, casename))
    remove("{}{}/{}.asm".format(CASE_DIR, casename, casename))
