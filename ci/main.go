// Dagger CI/CD pipeline for upgo project.
//
// Entrypoints:
//
//	dagger run go run . ci       — cargo check + cargo nextest
//	dagger run go run . check    — cargo check only
//	dagger run go run . test     — cargo nextest only
package main

import (
	"context"
	"fmt"
	"os"

	"dagger.io/dagger"
)

func main() {
	ctx := context.Background()

	client, err := dagger.Connect(ctx, dagger.WithLogOutput(os.Stderr))
	if err != nil {
		panic(err)
	}
	defer client.Close()

	entrypoint := "ci"
	if len(os.Args) > 1 {
		entrypoint = os.Args[1]
	}

	// Get source directory excluding artifacts
	src := client.Host().Directory(".", dagger.HostDirectoryOpts{
		Exclude: []string{"target", "node_modules", ".git", ".dagger", "ci"},
	})

	// Mount source into a Rust container and run commands
	rust := client.Container().From("rust:1.85-slim").
		WithMountedDirectory("/src", src).
		WithWorkdir("/src")

	var out string
	switch entrypoint {
	case "check":
		out, err = rust.WithExec([]string{"cargo", "check"}).Stdout(ctx)
	case "test":
		out, err = rust.WithExec([]string{"cargo", "nextest", "run"}).Stdout(ctx)
	default:
		out, err = rust.
			WithExec([]string{"cargo", "check"}).
			WithExec([]string{"cargo", "nextest", "run"}).Stdout(ctx)
	}

	if err != nil {
		fmt.Fprintf(os.Stderr, "Pipeline failed: %v\n", err)
		os.Exit(1)
	}
	fmt.Print(out)
}
