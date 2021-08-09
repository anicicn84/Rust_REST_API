In order to run the app, first you need to build it by going to the main folder where the src/ folder is and Cargo.toml file is present and from there run from the console `cargo build` command. 
It will download all of the necessary packages defined in the .toml file needed to be used for this project. 
Of course, you need to have rust compiler installed on your machine, depending on your OS.
https://www.rust-lang.org/tools/install

After the build a new target/ folder will be created.

Server side:

For running the app go to the root project's folder and run the following:
`cargo run --bin server -- target/debug/files`

** The folder files/ would be created in target/debug/ and you will have your uploaded files there. When you send DELETE request, the file will be deleted, also when you call list_files, you will get the paths to all of the files there. 

Client side:

In parallel bash/batch window run one of the following commands:
1) cargo run --bin fs_store -- list_files
2) cargo run --bin fs_store -- upload_file Cargo.toml
3) cargo run --bin fs_store -- delete_file Cargo.toml

You can also run these by going to target/debug folder and you will have server and fs_store executables which accept params:
./server files
./fs_store list_files
./fs_store upload_file Cargo.toml
./fs_store delete_file Cargo.toml

The 1st approach is easier, by running it from the root folder, not having to worry where the executables are.

The reason why Cargo.toml file is given here is because it is present in the running folder. Of course, you can specify another path to the file you want to upload, like for ex. on MAC /Users/User/Documents/file.txt




Test are not covered and improvement is about to come.