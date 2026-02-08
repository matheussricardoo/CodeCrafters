package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
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
				path_var := os.Getenv("PATH")
				found := false
				for _, dir := range filepath.SplitList(path_var) {
					command := filepath.Join(dir, list[1])
					info, err := os.Stat(command)
					if err == nil {
						mode := info.Mode()
						if mode.Perm()&0111 != 0 {
							fmt.Printf("%s is %s\n", list[1], command)
							found = true
							break
						}
					}
				}
				if !found {
					fmt.Printf("%s: not found\n", list[1])
				}
			}
		default:
			cmd := exec.Command(list[0], list[1:]...)
			cmd.Stderr = os.Stderr
			cmd.Stdout = os.Stdout
			err := cmd.Run()
			if err != nil {
				fmt.Printf("%s: command not found\n", list[0])
			}
		}
	}
}
