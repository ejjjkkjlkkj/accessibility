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

// validDNSDomain accepts an ASCII DNS name (including a fully-qualified trailing dot).
// Internationalized names must be supplied in their IDNA ASCII form (punycode).
func validDNSDomain(raw string) bool {
	domain := strings.TrimSuffix(strings.ToLower(strings.TrimSpace(raw)), ".")
	if domain == "" || len(domain) > 253 || net.ParseIP(domain) != nil {
		return false
	}
	for _, label := range strings.Split(domain, ".") {
		if len(label) == 0 || len(label) > 63 || label[0] == '-' || label[len(label)-1] == '-' {
			return false
		}
		for _, ch := range label {
			if !((ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9') || ch == '-') {
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
		if len(row) != len(header) {
			return fmt.Errorf("CSV line %d: got %d fields, want %d", line, len(row), len(header))
		}
		value := func(key string) string {
			return strings.TrimSpace(row[index[key]])
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
		parsedURL, err := url.Parse(value("url"))
		if err != nil || (strings.ToLower(parsedURL.Scheme) != "https" && strings.ToLower(parsedURL.Scheme) != "http") || parsedURL.Hostname() == "" || parsedURL.User != nil {
			return fmt.Errorf("CSV line %d: URL must be absolute HTTP(S), include a hostname, and contain no userinfo", line)
		}
		domain := strings.TrimSuffix(strings.ToLower(value("domain")), ".")
		hostname := strings.TrimSuffix(strings.ToLower(parsedURL.Hostname()), ".")
		if !validDNSDomain(domain) {
			return fmt.Errorf("CSV line %d: declared domain %q is not a valid DNS name", line, value("domain"))
		}
		if !validDNSDomain(hostname) {
			return fmt.Errorf("CSV line %d: URL hostname %q is not a valid DNS name", line, hostname)
		}
		if hostname != domain && !strings.HasSuffix(hostname, "."+domain) {
			return fmt.Errorf("CSV line %d: URL hostname %q does not match domain %q", line, hostname, domain)
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
