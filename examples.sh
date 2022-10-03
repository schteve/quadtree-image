#!/bin/sh

cargo run --release -- assets/input/dolphin.jpg -o assets/output/dolphin.png
cargo run --release -- assets/input/moon_stars.jpg -o assets/output/moon_stars.png -d 1000
cargo run --release -- assets/input/moon_stars.jpg -o assets/output/moon_stars_no_border.png -d 1000 --no-borders
cargo run --release -- assets/input/mountains.jpg -o assets/output/mountains.png -d 2000
cargo run --release -- assets/input/philly.jpg -o assets/output/philly.png -d 2000
cargo run --release -- assets/input/pluto.png -o assets/output/pluto.png
cargo run --release -- assets/input/sunflower.jpg -o assets/output/sunflower.png
cargo run --release -- assets/input/sunflower.jpg -o assets/output/sunflower_anim.png -alr
cargo run --release -- assets/input/tropical.jpg -o assets/output/tropical.png -d 2000
cargo run --release -- assets/input/wolf_sun.jpg -o assets/output/wolf_sun.png
