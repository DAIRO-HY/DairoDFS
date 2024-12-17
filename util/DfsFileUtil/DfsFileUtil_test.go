package DfsFileUtil

import (
	_ "embed"
	"log"
	"testing"
)

func TestContentTypeTxt(t *testing.T) {
	contentData, err := contentTypeTxt.ReadFile("content-type.txt")
	if err != nil {
		log.Fatal(err)
	}
	log.Println(contentData)
}
