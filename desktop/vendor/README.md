# Vendor npm dependency
The following instructions are based on this (blogpost)[https://stricker.digital/posts/how-to-vendor-npm-dependencies/].

```bash
git clone git@github.com/them/example-library.git
cd example-library
npm install
npm run build
npm pack <build-dir>
```

Move the `.tgz` file to `desktop/vendor` and install it.

```bash
cp example-library-1-0-0.tgz desktop/vendor/example-library-1-0-0.tgz
cd desktop/
npm install ./vendor/example-library-1-0-0.tgz
```
