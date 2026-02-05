package main

import (
	"fmt"
)

// Ensures gofmt doesn't remove the "fmt" import in stage 1 (feel free to remove this!)
var _ = fmt.Print

func main() {
	for {
		fmt.Print("$ ")
		var input string
		fmt.Scan(&input)
		if input == "exit" {
			break
		} else if input != "" {
			fmt.Printf("%s: command not found\n", input)
		}
	}
}
