cargo build
rmdir /S /Q dist
mkdir dist
mv target\debug\circumvention-chronicles.exe dist
robocopy /S assets dist/assets
butler -V
butler.exe push dist boringcactus/circumvention-chronicles:windows-jam

