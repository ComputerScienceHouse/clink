<center>
<img src="https://user-images.githubusercontent.com/48024900/194197157-5a8d9200-c66a-42d1-8ca5-782a81315469.png" alt="clink">
</center>

Clink is a command line interface for ordering drinks from any of the vending machines on floor.

It comes with a command line that can be used to list and purchase drinks.

![screenshot of a cli with a list of machines and their offerings](https://user-images.githubusercontent.com/42927786/157095410-d208a41d-adcb-4991-b9c1-3e7a8ed38f8f.png)

Or, if you prefer a more graphical experience, you can use the ncurses UI.

![screenshot of an ncurses selection menu for selecting a vending machine](https://user-images.githubusercontent.com/42927786/157095299-7c97a0a0-9bb7-4366-ba4f-94324189b950.png)

## Usage

To use clink, simply...

1. Log onto any user machine
2. Run `kinit` to get a kerberos token 
3. Run `clink`

![gif of clink in use](https://user-images.githubusercontent.com/42927786/157098855-302db1ed-13b8-4be5-b1bf-4431ea83e92f.gif)

## Development
```
git clone git@github.com/computersciencehouse/clink
cd clink
cargo build
```
