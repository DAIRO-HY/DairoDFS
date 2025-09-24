package DBBackupUtil

import (
	"testing"
)

func TestBackup(t *testing.T) {
	Backup()
	Backup()
}
func TestDeleteExpireFile(t *testing.T) {
	deleteExpireFile()
}
