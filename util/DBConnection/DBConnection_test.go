package DBConnection

import (
	"fmt"
	"testing"
	"time"
)

func TestTrascation(t *testing.T) {
	go func() {
		tx, _ := DBConn.Begin()
		var count int
		if _, err := tx.Exec("insert into user(id,name,date)values(123456,'name',123456)"); err != nil {
			fmt.Println(err)
			return
		}
		tx.QueryRow("select count(*) from user").Scan(&count)
		fmt.Println("---------------1")
		fmt.Println(count)
		fmt.Println("---------------1")
		time.Sleep(1000 * time.Second)
	}()

	go func() {
		time.Sleep(1 * time.Second)
		//tx, _ := DBConn.Begin()
		fmt.Println("---------------2")

		if _, err := DBConn.Exec("insert into user(id,name,date)values(2422,'bhjbj',123456)"); err != nil {
			fmt.Println("---------------2err")
			fmt.Println(err)
			fmt.Println("---------------2err")
			return
		}
		fmt.Println("---------------2")
		time.Sleep(1000 * time.Second)
	}()

	go func() {
		time.Sleep(2 * time.Second)
		tx, _ := DBConn.Begin()
		var count int
		tx.QueryRow("select count(*) from user").Scan(&count)
		fmt.Println("---------------3")
		fmt.Println(count)
		fmt.Println("---------------3")
		time.Sleep(1000 * time.Second)
	}()
	time.Sleep(1 * time.Hour)
}
