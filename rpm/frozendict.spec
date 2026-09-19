Name:           frozendict
Version:        2.1.0
Release:        1%{?dist}
Summary:        Blazingly fast, immutable dictionary Rust library
License:        MIT
URL:            https://github.com/wiseaidev/frozndict
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo

%description
frozendict provides a fully immutable, hashable key-value store backed by a
sorted contiguous heap allocation with O(log n) binary-search lookups and
O(1) pre-computed hashing. Native Python and Node.js bindings included.

%prep
%autosetup

%build
cargo build --release

%install
mkdir -p %{buildroot}%{_libdir}
cp target/release/libfrozendict.so %{buildroot}%{_libdir}/ 2>/dev/null || true

%files
%license LICENSE
%doc README.md
%{_libdir}/libfrozendict.so

%changelog
* Thu Sep 18 2026 Mahmoud Harmouch <oss@wiseai.dev> - 2.1.0-1
- Initial RPM release.
- Optimized FrozenMap (sort+dedup, multiplicative hash mixing).
- Node.js napi-rs bindings.
- Functional APIs: merge, with, without, intersection, union, difference.
