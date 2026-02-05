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
		default:
			fmt.Printf("%s: command not found\n", list[0])
		}
	}
}
