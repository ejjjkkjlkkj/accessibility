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

func TestValidateRejectsDuplicateHeader(t *testing.T) {
	input := "id,name,domain,url,category,layer,official_status,notes,id\n" +
		"a,Example,example.org,https://example.org/,docs,web,candidate,test,shadow\n"
	if err := validate(strings.NewReader(input)); err == nil || !strings.Contains(err.Error(), "duplicate header") {
		t.Fatalf("expected duplicate header error, got %v", err)
	}
}

func TestValidateRejectsMalformedAbsoluteURL(t *testing.T) {
	for _, value := range []string{"https://", "https:///path", "https://user:pass@example.org/"} {
		t.Run(value, func(t *testing.T) {
			row := "a,Example,example.org," + value + ",docs,web,candidate,test\n"
			if err := validate(strings.NewReader(validHeader + row)); err == nil {
				t.Fatalf("expected invalid URL %q to fail", value)
			}
		})
	}
}

func TestValidateRejectsWrongFieldCount(t *testing.T) {
	input := validHeader + "a,Example,example.org,https://example.org/,docs,web,candidate\n"
	if err := validate(strings.NewReader(input)); err == nil || !strings.Contains(err.Error(), "expected 8") {
		t.Fatalf("expected field count error, got %v", err)
	}
}
