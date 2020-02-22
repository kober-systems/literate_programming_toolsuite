

echo "Start generating source files ..."

cd lisa
lisa lisa.adoc
# The new generated source must be able to
# generate itself
cargo run --manifest-path ../Cargo.toml -- lisa.adoc || exit 1
cd ..

cargo run -- README.adoc || exit 1

echo "Generating source files done!"
cargo test || exit 1


while true; do
    read -p "Do you wish to commit your changes to git? [yes|no] " yn
    case $yn in
        [Yy]* )
          git diff;
          git add .;
          git commit;
          break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done

while true; do
    read -p "Do you wish to install this program? [yes|no] " yn
    case $yn in
        [Yy]* ) cargo install --force --path .; break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done
