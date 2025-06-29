# Zip extract with encoding test

## Why I write this program

I had extracted some old zip files with some compress/uncompress softwares, and I got messy code in file name and folder name. I found the reason is:

The elder version of zip file fromat does not contain the encoding of the file path, so when uncompressing a zip file that compressed on a machine with a different encoding of yours, you got messy code.

This bug had been solved in the later zip file fromat by storing the encoding of path into the zip file, but file compressed with the elder zip format still not well handled. 

So I write this program to help extract the zip file in the elder zip format.

## What this program do

The program will try to docode file path with all encodings untill find the right one, or it gives an error.

The program takes 2 arguments:
* 1st is path of zip file
* 2ed is folder to place the extracted files

The 2ed argument can be empty, uncompressed file would be placed in the parent folder of zip file without the 2ed argument.

Example:

`zip_encode_test a.zip ./foo/`

## Contribute

Feel free to send a pull request.