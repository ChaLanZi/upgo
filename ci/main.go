// Dagger CI/CD pipeline for upgo.
//
// Runs inside an isolated rust:slim container (tracks latest stable).
// For local development with Docker access, use `dagger run sh -c` instead.
//
// Usage:
//
//	dagger run go run . ci           — cargo check + cargo nextest (default)
//	dagger run go run . check        — cargo check only
//	dagger run go run . test         — cargo nextest only
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

	// -----------------------------------------------------------------------
	// Source directory (exclude local artifacts; ci/ is excluded because it
	// contains only the Dagger pipeline itself, not needed inside the build)
	// -----------------------------------------------------------------------
	src := client.Host().Directory(".", dagger.HostDirectoryOpts{
		Exclude: []string{"target", "node_modules", ".git", ".dagger", "ci"},
	})

	rust := client.Container().From("rust:slim").
		WithMountedDirectory("/src", src).
		WithWorkdir("/src")

	// Apply the same cfg flag used by the shell pipeline so that
	// #[cfg(docker_tests)] code is compiled and validated.
	// Run-time execution of Docker-based tests requires mounting the
	// Docker socket — see the `--with-docker` variant.
	rust = rust.WithEnvVariable("RUSTFLAGS", "--cfg docker_tests")

	var out string
	switch entrypoint {
	case "check":
		out, err = rust.WithExec([]string{"cargo", "check"}).Stdout(ctx)
	case "test":
		out, err = rust.WithExec([]string{"cargo", "nextest", "run"}).Stdout(ctx)
	default: // ci
		out, err = rust.
			WithExec([]string{"cargo", "check"}).
			WithExec([]string{"cargo", "nextest", "run"}).Stdout(ctx)
	}

	if err != nil {
		fmt.Fprintf(os.Stderr, "✗ Pipeline failed: %v\n", err)
		os.Exit(1)
	}
	fmt.Print(out)
}
