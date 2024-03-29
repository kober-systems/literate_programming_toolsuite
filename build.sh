set -e

changed_files=`git diff --name-only | grep --invert-match '\.adoc$'`
if [[ ! -z "$changed_files" ]]; then
  while true; do
      echo "some files are modified"
      echo "$changed_files"
      read -p "Do you wish continue anyway? [yes|no] " yn
      case $yn in
          [Yy]* ) break;;
          [Nn]* ) exit;;
          * ) echo "Please answer yes or no.";;
      esac
  done
fi

echo "Start generating source files ..."

cd asciidoctrine/
lisi -o ../docs/asciidoctrine/asciidoctrine.lisi.html asciidoctrine.adoc \
  || echo "lisi is currenty not installed"
cd ..

cd lisi
lisi -o /dev/null lisi.adoc || echo "lisi is currenty not installed"
# The new generated source must be able to
# generate itself
cargo run --manifest-path ../Cargo.toml --bin lisi -- -o lisi.html lisi.adoc
cd ..

cargo run --bin lisi -- -o /dev/null README.adoc

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
            -D docs/lisi \
            lisi/lisi.adoc
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
asciidoctor \
            -r asciidoctor-diagram \
            -a source-highlighter=pygments \
            -a toc=left \
            -a icons=font \
            -a toclevels=4 \
            -a data-uri \
            -a reproducible \
            -D docs/dolmetscher \
            dolmetscher/dolmetscher.adoc

echo "Generating html files done!"


while true; do
    git diff;
    read -p "Do you wish to commit your changes to git? [yes|no] " yn
    case $yn in
        [Yy]* )
          git add .;
          git commit;
          break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done

cargo publish --dry-run -p asciidoctrine

while true; do
    read -p "Do you wish to publish asciidoctrine? [yes|no] " yn
    case $yn in
        [Yy]* )
          cargo login;
          cargo publish --dry-run -p asciidoctrine;
          break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done

cargo publish --dry-run -p lisi

while true; do
    read -p "Do you wish to publish lisi? [yes|no] " yn
    case $yn in
        [Yy]* )
          cargo login;
          cargo publish --dry-run -p lisi;
          break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done

while true; do
    read -p "Do you wish to install this program? [yes|no] " yn
    case $yn in
        [Yy]* )
          nix-env -i -f lisi.nix;
          break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done
