package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"os"
	"strings"
)

var required = []string{"id", "name", "domain", "url", "category", "layer", "official_status", "notes"}

func validate(r io.Reader) error {
	reader := csv.NewReader(r)
	reader.FieldsPerRecord = -1
	header, err := reader.Read()
	if err != nil {
		return fmt.Errorf("read header: %w", err)
	}
	index := make(map[string]int, len(header))
	for i, name := range header {
		name = strings.TrimSpace(name)
		if name == "" {
			return fmt.Errorf("header column %d is empty", i+1)
		}
		if _, exists := index[name]; exists {
			return fmt.Errorf("duplicate header column %q", name)
		}
		index[name] = i
	}
	reader.FieldsPerRecord = len(header)
	for _, name := range required {
		if _, ok := index[name]; !ok {
			return fmt.Errorf("missing required column %q", name)
		}
	}
	seen := map[string]bool{}
	line := 1
	for {
		row, err := reader.Read()
		if err == io.EOF {
			break
		}
		line++
		if err != nil {
			return fmt.Errorf("CSV line %d: %w", line, err)
		}
		value := func(key string) string {
			i := index[key]
			if i >= len(row) {
				return ""
			}
			return strings.TrimSpace(row[i])
		}
		id := value("id")
		if id == "" {
			return fmt.Errorf("CSV line %d: empty id", line)
		}
		if seen[id] {
			return fmt.Errorf("CSV line %d: duplicate id %q", line, id)
		}
		seen[id] = true
		for _, key := range []string{"name", "domain", "url", "category", "layer", "official_status"} {
			if value(key) == "" {
				return fmt.Errorf("CSV line %d: empty %s", line, key)
			}
		}
		url := value("url")
		if !strings.HasPrefix(url, "https://") && !strings.HasPrefix(url, "http://") {
			return fmt.Errorf("CSV line %d: URL must use HTTP or HTTPS", line)
		}
	}
	if len(seen) == 0 {
		return fmt.Errorf("catalog has no resource rows")
	}
	fmt.Printf("PASS: validated %d resource rows\n", len(seen))
	return nil
}

func main() {
	path := "data/resources.csv"
	if len(os.Args) > 1 {
		path = os.Args[1]
	}
	file, err := os.Open(path)
	if err != nil {
		fmt.Fprintln(os.Stderr, "FAIL:", err)
		os.Exit(1)
	}
	defer file.Close()
	if err := validate(file); err != nil {
		fmt.Fprintln(os.Stderr, "FAIL:", err)
		os.Exit(1)
	}
}
