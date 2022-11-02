set -e

echo "Start generating source files ..."

cd lisa
lisa lisa.adoc
# The new generated source must be able to
# generate itself
cargo run --manifest-path ../Cargo.toml --bin lisa -- lisa.adoc
cd ..

cargo run --bin lisa -- -o /dev/null README.adoc

echo "Generating source files done!"
cargo test
echo "Start generating html files ..."

asciidoctor \
            -r asciidoctor-diagram \
            -a source-highlighter=pygments \
            -a toc=left \
            -a icons=font \
            -a toclevels=4 \
            -a data-uri \
            -a reproducible \
            -D docs \
            README.adoc -o index.html
asciidoctor \
            -r asciidoctor-diagram \
            -a source-highlighter=pygments \
            -a toc=left \
            -a icons=font \
            -a toclevels=4 \
            -a data-uri \
            -a reproducible \
            -D docs/lisa \
            lisa/lisa.adoc
asciidoctor \
            -r asciidoctor-diagram \
            -a source-highlighter=pygments \
            -a toc=left \
            -a icons=font \
            -a toclevels=4 \
            -a data-uri \
            -a reproducible \
            -D docs/asciidoctrine \
            asciidoctrine/asciidoctrine.adoc
asciidoctor \
            -r asciidoctor-diagram \
            -a source-highlighter=pygments \
            -a toc=left \
            -a icons=font \
            -a toclevels=4 \
            -a data-uri \
            -a reproducible \
            -D docs/ansicht \
            ansicht/ansicht.adoc

echo "Generating html files done!"


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
        [Yy]* ) cargo install --force --path lisa; break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done
