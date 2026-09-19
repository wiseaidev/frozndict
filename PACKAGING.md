# frozendict Packaging Guide (Debian / RPM)

This document explains how to build and install native `.deb` and `.rpm`
packages for the `frozendict` Rust binary and library.

> [!NOTE]
> Debian and RPM packages bundle the `frozendict` Rust library only. The
> Python (`frozndict`) and Node.js (`frozendict`) packages are distributed
> separately via PyPI and npm.

## 🏗 Debian / Ubuntu

### Prerequisites

```sh
sudo apt-get install -y build-essential debhelper devscripts pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build the `.deb`

```sh
git clone https://github.com/wiseaidev/frozndict.git
cd frozndict
debuild -d --preserve-envvar PATH -us -uc -b
ls ../*.deb
```

### Install

```sh
sudo dpkg -i ../frozendict_2.1.1_amd64.deb
```

### Verify

```sh
dpkg -s frozendict
```

## 🏗 RHEL / Fedora

### Prerequisites

```sh
sudo dnf install -y rpm-build gcc openssl-devel pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Prepare the RPM environment

```sh
mkdir -p ~/rpmbuild/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}
cp rpm/frozendict.spec ~/rpmbuild/SPECS/

VERSION=$(grep -m1 '^version =' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

tar -czvf ~/rpmbuild/SOURCES/frozendict-${VERSION}.tar.gz \
  --transform "s,^\.,frozendict-${VERSION}," \
  --exclude=.git .
```

### Build the `.rpm`

```sh
rpmbuild -bb ~/rpmbuild/SPECS/frozendict.spec
ls ~/rpmbuild/RPMS/x86_64/
```

### Install

```sh
sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/frozendict-2.1.1-1.x86_64.rpm
```

### Verify

```sh
rpm -qi frozendict
```

## 📦 Pre-built Packages (GitHub Releases)

Pre-built `.deb` and `.rpm` packages are attached to every
[GitHub Release](https://github.com/wiseaidev/frozndict/releases).

Download with `gh`:

```sh
gh release download v2.1.1 --pattern '*.deb'
gh release download v2.1.1 --pattern '*.rpm'
```

## 🔗 See Also

- [Debian Policy Manual](https://www.debian.org/doc/debian-policy/)
- [RPM Packaging Guide](https://rpm-packaging-guide.github.io/)
- [linux-publish.yml](https://github.com/wiseaidev/frozndict/blob/main/.github/workflows/linux-publish.yml): CI workflow
