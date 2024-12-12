package resources

import (
	"embed"
)

//go:embed sql/create/*
var SqlFolder embed.FS
