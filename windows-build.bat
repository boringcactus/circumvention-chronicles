cargo build --release
mkdir dist
mv target\release\circumvention-chronicles.exe dist
copy assets dist\
butler -V
butler.exe push dist boringcactus/circumvention-chronicles:windows-jam

