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

func TestValidateRejectsEmptyCatalog(t *testing.T) {
	if err := validate(strings.NewReader(validHeader)); err == nil {
		t.Fatal("expected catalog without resource rows to fail")
	}
}

func TestValidateRejectsEmptyRequiredFields(t *testing.T) {
	rows := []string{
		",Example,example.org,https://example.org/,docs,web,candidate,test\n",
		"a,,example.org,https://example.org/,docs,web,candidate,test\n",
		"a,Example,,https://example.org/,docs,web,candidate,test\n",
		"a,Example,example.org,https://example.org/,,web,candidate,test\n",
		"a,Example,example.org,https://example.org/,docs,,candidate,test\n",
		"a,Example,example.org,https://example.org/,docs,web,,test\n",
	}
	for _, row := range rows {
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected empty required field to fail: %q", row)
		}
	}
}

func TestValidateRejectsMissingRequiredHeader(t *testing.T) {
	header := "id,name,domain,url,category,layer,official_status\n"
	row := "a,Example,example.org,https://example.org/,docs,web,candidate\n"
	if err := validate(strings.NewReader(header + row)); err == nil {
		t.Fatal("expected missing required header to fail")
	}
}

func TestValidateRejectsDuplicateHeadersAfterTrimming(t *testing.T) {
	header := "id,name, name ,domain,url,category,layer,official_status,notes\n"
	row := "a,Example,Shadow,example.org,https://example.org/,docs,web,candidate,test\n"
	if err := validate(strings.NewReader(header + row)); err == nil {
		t.Fatal("expected whitespace-normalized duplicate header to fail")
	}
}

func TestValidateRejectsOutOfRangeURLPorts(t *testing.T) {
	for _, port := range []string{"0", "65536"} {
		row := "a,Example,example.org,https://example.org:" + port + "/,docs,web,candidate,test\\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected invalid port %q to fail", port)
		}
	}
}

func TestValidateAcceptsValidURLPort(t *testing.T) {
	row := "a,Example,example.org,https://example.org:443/,docs,web,candidate,test\\n"
	if err := validate(strings.NewReader(validHeader + row)); err != nil {
		t.Fatalf("expected valid HTTPS port to pass, got %v", err)
	}
}

func TestValidateRejectsURLDomainMismatch(t *testing.T) {
	for _, test := range []struct {
		domain string
		rawURL string
	}{
		{domain: "example.org", rawURL: "https://evil.test/"},
		{domain: "example.org", rawURL: "https://example.org.evil.test/"},
	} {
		row := "a,Example," + test.domain + "," + test.rawURL + ",docs,web,candidate,test\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected domain mismatch to fail: domain=%q url=%q", test.domain, test.rawURL)
		}
	}
}

func TestValidateAcceptsSubdomainOfDeclaredDomain(t *testing.T) {
	row := "a,Example,example.org,https://docs.example.org/,docs,web,candidate,test\n"
	if err := validate(strings.NewReader(validHeader + row)); err != nil {
		t.Fatalf("expected valid subdomain URL, got %v", err)
	}
}

func TestValidateEnforcesDNSDomainLengthBoundary(t *testing.T) {
	makeDomain := func(lastLabelLength int) string {
		return strings.Join([]string{
			strings.Repeat("a", 63),
			strings.Repeat("b", 63),
			strings.Repeat("c", 63),
			strings.Repeat("d", lastLabelLength),
		}, ".")
	}

	maximum := makeDomain(61) // 253 octets including separators
	row := "a,Example," + maximum + ",https://" + maximum + "/,docs,web,candidate,test\\n"
	if err := validate(strings.NewReader(validHeader + row)); err != nil {
		t.Fatalf("expected 253-character DNS name to pass, got %v", err)
	}

	tooLong := makeDomain(62) // 254 octets including separators
	row = "a,Example," + tooLong + ",https://" + tooLong + "/,docs,web,candidate,test\\n"
	if err := validate(strings.NewReader(validHeader + row)); err == nil {
		t.Fatal("expected 254-character DNS name to fail")
	}
}


func TestValidateRequiresIDNAASCIIForInternationalizedDomains(t *testing.T) {
	t.Run("punycode is accepted", func(t *testing.T) {
		domain := "xn--bcher-kva.example"
		row := "a,Example," + domain + ",https://" + domain + "/,docs,web,candidate,test\n"
		if err := validate(strings.NewReader(validHeader + row)); err != nil {
			t.Fatalf("expected IDNA ASCII form to pass, got %v", err)
		}
	})

	t.Run("raw Unicode is rejected", func(t *testing.T) {
		domain := "bücher.example"
		row := "a,Example," + domain + ",https://" + domain + "/,docs,web,candidate,test\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Fatal("expected non-ASCII domain to be rejected; catalog requires IDNA ASCII form")
		}
	})
}

func TestValidateRejectsMalformedDeclaredDomain(t *testing.T) {
	for _, domain := range []string{"example..org", "-example.org", "example-.org", "exa_mple.org", "example.org/path", strings.Repeat("a", 64) + ".org"} {
		row := "a,Example," + domain + ",https://example.org/,docs,web,candidate,test\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected malformed declared domain to fail: %q", domain)
		}
	}
}

func TestValidateRejectsIPLiteralAsDomain(t *testing.T) {
	for _, host := range []string{"127.0.0.1", "192.168.1.10"} {
		row := "a,Example," + host + ",https://" + host + "/,docs,web,candidate,test\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected IP literal to be rejected as a DNS domain: %q", host)
		}
	}
}

func TestValidateRejectsIPLiteralAsURLHostname(t *testing.T) {
	for _, rawURL := range []string{"https://127.0.0.1/", "http://192.168.1.10/"} {
		row := "a,Example,example.org," + rawURL + ",docs,web,candidate,test\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected IP URL hostname to be rejected: %q", rawURL)
		}
	}
}

func TestValidateAcceptsCaseInsensitiveFullyQualifiedDomain(t *testing.T) {
	row := "a,Example,Example.ORG.,https://docs.example.org./,docs,web,candidate,test\n"
	if err := validate(strings.NewReader(validHeader + row)); err != nil {
		t.Fatalf("expected normalized DNS domain to pass, got %v", err)
	}
}

func TestValidateRejectsMalformedURLHostnames(t *testing.T) {
	for _, rawURL := range []string{
		"https://bad_host.example.org/",
		"https://example..org/",
		"https://-bad.example.org/",
		"https://bad-.example.org/",
		"https://[::1]/",
	} {
		row := "a,Example,example.org," + rawURL + ",docs,web,candidate,test\\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected invalid DNS hostname to fail: %q", rawURL)
		}
	}
}

func TestValidateRejectsMalformedURL(t *testing.T) {
	for _, rawURL := range []string{"https://%", "http://[::1", "https://?query=missing-host"} {
		row := "a,Example,example.org," + rawURL + ",docs,web,candidate,test\n"
		if err := validate(strings.NewReader(validHeader + row)); err == nil {
			t.Errorf("expected malformed URL to fail: %q", rawURL)
		}
	}
}
