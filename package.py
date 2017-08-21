#!/usr/bin/python

# This is a temporary solution. We eventually want to add these features to cargo-bundle

import os
import os.path as path
import shutil
import subprocess
import sys

def otool(s):
    o = subprocess.Popen(['/usr/bin/otool', '-L', s], stdout=subprocess.PIPE)
    for l in o.stdout:
        if l[0] == '\t':
            yield l.split(' ', 1)[0][1:]


def install_name_tool(old, new, binary):
    try:
        subprocess.check_call(['install_name_tool', '-change', old, '@executable_path/' + new, binary])
    except subprocess.CalledProcessError as e:
        print("install_name_tool exited with return value %d" % e.returncode)


def is_system_library(lib):
    return lib.startswith("/System/Library") or lib.startswith("/usr/lib")


def change_non_system_libraries_path(libraries, relative_path, binary):
    for lib in libraries:
        if is_system_library(lib):
            continue
        new_path = path.join(relative_path, path.basename(lib))
        install_name_tool(lib, new_path, binary)


def package(target):
    contents = 'target/' + target + '/ServoShell.app/Contents/'

    framework_path = contents + "Frameworks"
    shutil.rmtree(framework_path, True)
    os.makedirs(framework_path)
    shutil.copytree("./target/MMTabBarView/Release/MMTabBarView.framework", framework_path + "/MMTabBarView.framework")

    lib_path = contents + "MacOS/"
    binary_path = lib_path + "servoshell"

    relative_path = path.relpath(lib_path, path.dirname(binary_path)) + "/"

    binary_dependencies = [lib for lib in set(otool(binary_path)) if not lib.startswith("@rpath")]

    change_non_system_libraries_path(binary_dependencies, relative_path, binary_path)

    need_checked = binary_dependencies
    checked = set()
    while need_checked:
        checking = set(need_checked)
        need_checked = set()
        for f in checking:
            # No need to check these for their dylibs
            if is_system_library(f):
                continue
            need_relinked = set(otool(f))
            new_path = path.join(lib_path, path.basename(f))
            if not path.exists(new_path):
                shutil.copyfile(f, new_path)
            change_non_system_libraries_path(need_relinked, relative_path, new_path)
            need_checked.update(need_relinked)
        checked.update(checking)
        need_checked.difference_update(checked)
    

    dmg_path = 'target/' + target + '/ServoShell.dmg'
    try:
        os.remove(dmg_path)
    except OSError:
        pass
    dmg_tmp_path = 'target/' + target + '/ServoShell_tmp/'
    dot_app_path = 'target/' + target + '/ServoShell.app/'
    os.makedirs(dmg_tmp_path)
    shutil.copytree(dot_app_path, dmg_tmp_path  + "ServoShell.app/")
    subprocess.check_call(['hdiutil', 'create', '-volname', 'Servo', '-megabytes', '900', dmg_path, '-srcfolder', dmg_tmp_path])
    shutil.rmtree(dmg_tmp_path, True)


if len(sys.argv) > 1 and sys.argv[1] == "--release":
    package('release')
else:
    package('debug')
