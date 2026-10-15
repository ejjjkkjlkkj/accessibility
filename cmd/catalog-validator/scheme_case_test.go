package main

import (
	"strings"
	"testing"
)

func TestValidateAcceptsCaseInsensitiveHTTPScheme(t *testing.T) {
	for _, rawURL := range []string{
		"HTTPS://example.org/",
		"HtTpS://docs.example.org/",
		"HTTP://example.org/",
	} {
		t.Run(rawURL, func(t *testing.T) {
			row := "a,Example,example.org," + rawURL + ",docs,web,candidate,test\n"
			if err := validate(strings.NewReader(validHeader + row)); err != nil {
				t.Fatalf("expected valid case-insensitive HTTP scheme, got %v", err)
			}
		})
	}
}
