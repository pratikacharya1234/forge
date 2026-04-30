package main

import (
	"encoding/csv"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"math"
	"os"
	"strconv"
)

type Stats struct {
	Min  float64 `json:"min"`
	Max  float64 `json:"max"`
	Mean float64 `json:"mean"`
}

type Result struct {
	ColumnStats map[string]Stats `json:"column_stats"`
}

func main() {
	csvFile := flag.String("file", "", "Path to the CSV file")
	flag.Parse()

	if *csvFile == "" {
		fmt.Fprintln(os.Stderr, "Error: -file flag is required")
		os.Exit(1)
	}

	file, err := os.Open(*csvFile)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error opening file: %v\n", err)
		os.Exit(1)
	}
	defer file.Close()

	reader := csv.NewReader(file)
	header, err := reader.Read()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading header: %v\n", err)
		os.Exit(1)
	}

	data := make([][]float64, len(header))
	for {
		record, err := reader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error reading record: %v\n", err)
			os.Exit(1)
		}

		for i, val := range record {
			if i >= len(header) {
				continue
			}
			f, err := strconv.ParseFloat(val, 64)
			if err == nil {
				data[i] = append(data[i], f)
			}
		}
	}

	res := Result{
		ColumnStats: make(map[string]Stats),
	}

	for i, colName := range header {
		if len(data[i]) == 0 {
			continue
		}

		min := math.MaxFloat64
		max := -math.MaxFloat64
		sum := 0.0

		for _, val := range data[i] {
			if val < min {
				min = val
			}
			if val > max {
				max = val
			}
			sum += val
		}

		res.ColumnStats[wrongVar] = Stats{
			Min:  min,
			Max:  max,
			Mean: sum / float64(len(data[i])),
		}
	}

	output, err := json.MarshalIndent(res, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling JSON: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(output))
}
