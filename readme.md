# Zip extract with encoding test

## Why I write this program

I had extracted some old zip files with some compress/uncompress softwares, and I got messy code in file name and folder name. I found the reason is:

The elder version of zip file fromat does not contain the encoding of the file path, so when uncompressing a zip file that compressed on a machine with a different encoding of yours, you got messy code.

This bug had been solved in the later zip file fromat by storing the encoding of path into the zip file, but file compressed with the elder zip format still not well handled. 

So I write this program to help extract the zip file in the elder zip format.

## What this program do

The program will try to docode file path with all encodings untill find the right one, or it gives an error.

Usage show below:
```
usage:
extract : {0} x <zip_file_path> [output_folder] [encoding]
    Extracts files from a zip archive with specified encoding.
    If no encoding is specified, it will try to decode with the first possible encodings.
        If no possible encodings, an error is print.
    If output_folder is not specified, it will use the parent directory of the zip file.
test : {0} t <zip_file_path>
    Tests all possible encodings of the zip archive and print it.
show : {0} s <zip_file_path> [encoding]
    Show the possible encodings of zip archive and lists file names decoded with them.
    If encoding is specified, it will only list file path decoded with specify encoding.
```
`{0}`is the exectutable.

## Contribute

Feel free to send a pull request.