set -xo

cd ui/
ng build --prod --deploy-url /static/ --output-path dist/
cd ../
cargo build