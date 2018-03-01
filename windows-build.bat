cargo build --release
rmdir /S /Q dist
mkdir dist
mv target\release\circumvention-chronicles.exe dist
robocopy /S assets dist/assets
butler -V
butler.exe push dist boringcactus/circumvention-chronicles:windows-jam

