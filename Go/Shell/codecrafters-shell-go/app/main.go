package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func main() {
	for {
		fmt.Print("$ ")
		scanner := bufio.NewScanner(os.Stdin)
		scanner.Scan()
		input := scanner.Text()
		list := strings.Fields(input)
		if len(list) == 0 {
			continue
		}
		switch list[0] {
		case "exit":
			os.Exit(0)
		case "echo":
			args := list[1:]
			total_args := strings.Join(args, " ")
			fmt.Println(total_args)
		case "type":
			if list[1] == "exit" || list[1] == "echo" || list[1] == "type" {
				fmt.Printf("%s is a shell builtin\n", list[1])
			} else {
				fmt.Printf("%s: not found\n", list[1])
			}
		default:
			fmt.Printf("%s: command not found\n", list[0])
		}
	}
}
