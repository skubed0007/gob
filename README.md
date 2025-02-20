# Gob: The Universal Package Manager for All Linux Distributions

### ***Welcome to Gob, the gobbling package manager designed to unify and simplify package management across all Linux distributions!***
---
**Gob** isn't your run-of-the-mill Linux package manager. It's a native Linux package manager that fetches binaries directly from the source and installs them for you. No extra runtime bloat, no unnecessary dependencies—just pure, unadulterated efficiency. It just works!

---
install gob with the following command:
```bash
curl -sSfL https://raw.githubusercontent.com/skubed0007/gob/main/install.sh | sh
```
after installing gob please run the following command as your first command!
```bash
gob update
```
---
gob commands are as following:

- ```gob search <package_names_sep_by_space>```  : searches for packages having the match

- ``gob about <package_names_sep_by_space>`` : get information about package(s)

- ``gob install <package_names_sep_by_space>"`` : install the package 

---
## Upcoming commands
- ``gob local <path_to_package>`` : install a package from a zip file / binary file
- ``gob remove <package(s) name>``: remove/uninstall packages