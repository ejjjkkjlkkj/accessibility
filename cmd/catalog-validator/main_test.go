package main

import (
	"strings"
	"testing"
)

const validHeader = "id,name,domain,url,category,layer,official_status,notes\n"
const validRow = "a,Example,example.org,https://example.org/,docs,web,candidate,test\n"

func TestValidateAcceptsValidCatalog(t *testing.T) {
	if err := validate(strings.NewReader(validHeader + validRow)); err != nil {
		t.Fatalf("expected valid catalog, got %v", err)
	}
}

func TestValidateRejectsDuplicateIDs(t *testing.T) {
	input := validHeader + validRow + "a,Other,other.org,https://other.org/,docs,web,candidate,test\n"
	if err := validate(strings.NewReader(input)); err == nil {
		t.Fatal("expected duplicate ID to fail")
	}
}

func TestValidateRejectsNonHTTPURL(t *testing.T) {
	input := validHeader + "a,Example,example.org,file:///tmp,docs,web,candidate,test\n"
	if err := validate(strings.NewReader(input)); err == nil {
		t.Fatal("expected invalid URL scheme to fail")
	}
}

func TestValidateRejectsDuplicateHeaderColumns(t *testing.T) {
	header := "id,name,domain,url,category,layer,official_status,notes,name\n"
	row := "a,Example,example.org,https://example.org/,docs,web,candidate,test,shadow\n"
	if err := validate(strings.NewReader(header + row)); err == nil {
		t.Fatal("expected duplicate header column to fail")
	}
}

func TestValidateRejectsEmptyHeaderColumn(t *testing.T) {
	header := "id,name,domain,url,category,layer,official_status,\n"
	if err := validate(strings.NewReader(header + validRow)); err == nil {
		t.Fatal("expected empty header column to fail")
	}
}

func TestValidateRejectsMissingAndExtraFields(t *testing.T) {
	for _, row := range []string{
		"a,Example,example.org,https://example.org/,docs,web,candidate\n",
		"a,Example,example.org,https://example.org/,docs,web,candidate,test,extra\n",
	} {
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Fatalf("expected malformed record to fail: %q", row)
		}
	}
}

func TestValidateRejectsHTTPURLWithoutHostname(t *testing.T) {
	for _, rawURL := range []string{"https://", "http:///path", "https://user:pass@example.org/"} {
		row := "a,Example,example.org," + rawURL + ",docs,web,candidate,test\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected invalid URL to fail: %q", rawURL)
		}
	}
}
