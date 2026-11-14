Name:           glorious-launcher
Version:        0.1.10
Release:        1%{?dist}
Summary:        Web GUI and HID service for Glorious gaming mice
License:        MIT
BuildArch:      x86_64

Source0:        glorious-service
Source1:        glorious_devices.json
Source2:        glorious-launcher.desktop
Source3:        glorious-launcher.svg
Source4:        glorious-launcher.service
Source5:        99-glorious.rules
Source6:        LICENSE

%description
Web GUI and HID service for configuring Glorious gaming mice.

%prep

%build

%install
install -Dpm755 %{SOURCE0} %{buildroot}/usr/bin/glorious-service
install -Dpm644 %{SOURCE1} \
    %{buildroot}/usr/share/glorious-launcher/data/glorious_devices.json
install -Dpm644 %{SOURCE2} \
    %{buildroot}/usr/share/applications/glorious-launcher.desktop
install -Dpm644 %{SOURCE3} \
    %{buildroot}/usr/share/icons/hicolor/scalable/apps/glorious-launcher.svg
install -Dpm644 %{SOURCE4} \
    %{buildroot}/usr/lib/systemd/user/glorious-launcher.service
install -Dpm644 %{SOURCE5} \
    %{buildroot}/usr/lib/udev/rules.d/99-glorious.rules
install -Dpm644 %{SOURCE6} \
    %{buildroot}/usr/share/doc/glorious-launcher/copyright

%post
udevadm control --reload-rules || true
udevadm trigger || true

%postun
udevadm control --reload-rules || true
udevadm trigger || true

%files
/usr/bin/glorious-service
/usr/share/glorious-launcher/data/glorious_devices.json
/usr/share/applications/glorious-launcher.desktop
/usr/share/icons/hicolor/scalable/apps/glorious-launcher.svg
/usr/lib/systemd/user/glorious-launcher.service
/usr/lib/udev/rules.d/99-glorious.rules
/usr/share/doc/glorious-launcher/copyright

%changelog
* Sat Nov 14 2026 Glorious Web Launcher contributors - 0.1.9-1
- Align tray battery color tiers with Glorious manual and fix graphical-session launcher startup

* Mon Nov 09 2026 Glorious Web Launcher contributors - 0.1.8-1
- Add RPM packaging for Linux
