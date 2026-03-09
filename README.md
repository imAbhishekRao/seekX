# seekX

## Clone and run

```bash
# Arch Linux
sudo pacman -S --needed rust cargo gtk4 gtk4-layer-shell

git clone https://github.com/aman7935/seekX.git seekX
cd seekX
cargo run --features layer-shell
```

## Add to Applications menu and terminal

Build once:

```bash
cd seekX
cargo build --release --features layer-shell
```

Create desktop entry:

```bash
mkdir -p ~/.local/share/applications
cat > ~/.local/share/applications/seekx.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=SeekX
Exec=/home/your-user/seekX/target/release/seekX
Terminal=false
Categories=Utility;
EOF
```

Replace `your-user` with your Linux username.

To run from terminal as `seekX`:

```bash
mkdir -p ~/.local/bin
ln -sf /home/$USER/seekX/target/release/seekX ~/.local/bin/seekX
```
