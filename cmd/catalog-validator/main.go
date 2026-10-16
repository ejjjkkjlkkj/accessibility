package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"net"
	"net/url"
	"os"
	"strings"
)

var required = []string{"id", "name", "domain", "url", "category", "layer", "official_status", "notes"}

func validDomain(domain string) bool {
	domain = strings.TrimSuffix(strings.ToLower(domain), ".")
	if domain == "" || len(domain) > 253 || strings.Contains(domain, "..") {
		return false
	}
	if net.ParseIP(domain) != nil {
		return true
	}
	for _, label := range strings.Split(domain, ".") {
		if len(label) == 0 || len(label) > 63 || label[0] == '-' || label[len(label)-1] == '-' {
			return false
		}
		for _, r := range label {
			if !((r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') || r == '-') {
				return false
			}
		}
	}
	return true
}

func validate(r io.Reader) error {
	reader := csv.NewReader(r)
	reader.FieldsPerRecord = -1
	header, err := reader.Read()
	if err != nil {
		return fmt.Errorf("read header: %w", err)
	}
	index := make(map[string]int, len(header))
	for i, rawName := range header {
		name := strings.TrimSpace(rawName)
		if name == "" {
			return fmt.Errorf("header column %d has an empty name", i+1)
		}
		if _, exists := index[name]; exists {
			return fmt.Errorf("duplicate header column %q", name)
		}
		index[name] = i
	}
	for _, name := range required {
		if _, ok := index[name]; !ok {
			return fmt.Errorf("missing required column %q", name)
		}
	}
	seen := map[string]bool{}
	record := 1
	for {
		row, err := reader.Read()
		if err == io.EOF {
			break
		}
		record++
		if err != nil {
			return fmt.Errorf("CSV record %d: %w", record, err)
		}
		if len(row) != len(header) {
			return fmt.Errorf("CSV record %d: got %d fields, want %d", record, len(row), len(header))
		}
		value := func(key string) string { return strings.TrimSpace(row[index[key]]) }
		id := value("id")
		if id == "" {
			return fmt.Errorf("CSV record %d: empty id", record)
		}
		if seen[id] {
			return fmt.Errorf("CSV record %d: duplicate id %q", record, id)
		}
		seen[id] = true
		for _, key := range []string{"name", "domain", "url", "category", "layer", "official_status"} {
			if value(key) == "" {
				return fmt.Errorf("CSV record %d: empty %s", record, key)
			}
		}
		domain := strings.TrimSuffix(strings.ToLower(value("domain")), ".")
		if !validDomain(domain) {
			return fmt.Errorf("CSV record %d: invalid DNS domain %q", record, value("domain"))
		}
		parsedURL, err := url.Parse(value("url"))
		if err != nil || (parsedURL.Scheme != "https" && parsedURL.Scheme != "http") || parsedURL.Hostname() == "" || parsedURL.User != nil {
			return fmt.Errorf("CSV record %d: URL must be absolute HTTP(S), include a hostname, and contain no userinfo", record)
		}
		hostname := strings.TrimSuffix(strings.ToLower(parsedURL.Hostname()), ".")
		if !validDomain(hostname) || (hostname != domain && !strings.HasSuffix(hostname, "."+domain)) {
			return fmt.Errorf("CSV record %d: URL hostname %q does not match valid declared domain %q", record, hostname, domain)
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
