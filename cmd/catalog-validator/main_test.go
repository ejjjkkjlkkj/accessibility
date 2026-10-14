package main

import (
	"strings"
	"testing"
)

func TestValidateAcceptsValidCatalog(t *testing.T) {
	input := "id,name,domain,url,category,layer,official_status,notes\na,Example,example.org,https://example.org/,docs,web,candidate,test\n"
	if err := validate(strings.NewReader(input)); err != nil {
		t.Fatalf("expected valid catalog, got %v", err)
	}
}

func TestValidateRejectsDuplicateIDs(t *testing.T) {
	input := "id,name,domain,url,category,layer,official_status,notes\na,Example,example.org,https://example.org/,docs,web,candidate,test\na,Other,other.org,https://other.org/,docs,web,candidate,test\n"
	if err := validate(strings.NewReader(input)); err == nil {
		t.Fatal("expected duplicate ID to fail")
	}
}

func TestValidateRejectsNonHTTPURL(t *testing.T) {
	input := "id,name,domain,url,category,layer,official_status,notes\na,Example,example.org,file:///tmp,docs,web,candidate,test\n"
	if err := validate(strings.NewReader(input)); err == nil {
		t.Fatal("expected invalid URL scheme to fail")
	}
}
