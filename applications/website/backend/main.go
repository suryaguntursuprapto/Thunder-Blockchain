package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
)

const rpcURL = "http://127.0.0.1:8080"

type RpcRequest struct {
	JsonRpc string      `json:"jsonrpc"`
	Method  string      `json:"method"`
	Params  interface{} `json:"params"`
	ID      int         `json:"id"`
}

func fetchRpc(method string, params interface{}) (map[string]interface{}, error) {
	if params == nil {
		params = make(map[string]interface{})
	}

	reqBody := RpcRequest{
		JsonRpc: "2.0",
		Method:  method,
		Params:  params,
		ID:      1,
	}

	b, err := json.Marshal(reqBody)
	if err != nil {
		return nil, err
	}

	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Post(rpcURL, "application/json", bytes.NewBuffer(b))
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	var result map[string]interface{}
	if err := json.Unmarshal(bodyBytes, &result); err != nil {
		return nil, err
	}

	if result["result"] != nil {
		m, ok := result["result"].(map[string]interface{})
		if ok {
			return m, nil
		}
	}
	return nil, fmt.Errorf("rpc error or empty result")
}

func main() {
	app := fiber.New(fiber.Config{
		DisableStartupMessage: true,
	})

	app.Use(cors.New(cors.Config{
		AllowOrigins: "*",
	}))

	// /api/stats
	app.Get("/api/stats", func(c *fiber.Ctx) error {
		var wg sync.WaitGroup
		var hData, vData map[string]interface{}

		wg.Add(2)
		go func() {
			defer wg.Done()
			hData, _ = fetchRpc("thunder_blockNumber", nil)
		}()
		go func() {
			defer wg.Done()
			vData, _ = fetchRpc("thunder_getValidators", nil)
		}()
		wg.Wait()

		h := 0
		if hData != nil && hData["height"] != nil {
			h = int(hData["height"].(float64))
		}

		v := 0
		if vData != nil && vData["validators"] != nil {
			vals := vData["validators"].([]interface{})
			for _, val := range vals {
				vm := val.(map[string]interface{})
				if vm["is_active"] == true {
					v++
				}
			}
		}

		return c.JSON(fiber.Map{
			"blockHeight":      h,
			"activeValidators": v,
			"price":            1.24,
			"priceChange":      5.2,
			"totalStaked":      100000,
		})
	})

	// /api/blocks/latest
	app.Get("/api/blocks/latest", func(c *fiber.Ctx) error {
		hData, _ := fetchRpc("thunder_blockNumber", nil)
		currentHeight := 0
		if hData != nil && hData["height"] != nil {
			currentHeight = int(hData["height"].(float64))
		}

		limitStr := c.Query("limit", "10")
		maxBlocks, err := strconv.Atoi(limitStr)
		if err != nil {
			maxBlocks = 10
		}

		limit := currentHeight - maxBlocks
		if limit < 0 {
			limit = 0
		}

		var blocks []interface{}
		var transactions []interface{}

		for i := currentHeight; i >= limit; i-- {
			block, err := fetchRpc("thunder_getBlock", map[string]interface{}{"height": i})
			if err == nil && block != nil {
				if block["transactions"] == nil {
					block["transactions"] = []interface{}{}
				}
				blocks = append(blocks, block)
				txs := block["transactions"].([]interface{})
				transactions = append(transactions, txs...)
			}
		}

		if len(blocks) == 0 {
			blocks = append(blocks, fiber.Map{
				"height":       currentHeight,
				"hash":         "0x0000000000000000",
				"transactions": []interface{}{},
				"timestamp":    time.Now().Unix(),
				"validator":    "0x0000000000000000000000000000000000000000",
				"txn_count":    0,
			})
		}

		return c.JSON(fiber.Map{
			"blocks":       blocks,
			"transactions": transactions,
		})
	})

	// /api/block/:height
	app.Get("/api/block/:height", func(c *fiber.Ctx) error {
		heightStr := c.Params("height")
		h, err := strconv.Atoi(heightStr)
		if err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid height"})
		}

		block, _ := fetchRpc("thunder_getBlock", map[string]interface{}{"height": h})
		if block != nil {
			if block["transactions"] == nil {
				block["transactions"] = []interface{}{}
			}
			return c.JSON(block)
		}
		return c.Status(404).JSON(fiber.Map{"error": "Block not found"})
	})

	// /api/validators
	app.Get("/api/validators", func(c *fiber.Ctx) error {
		data, _ := fetchRpc("thunder_getValidators", nil)
		if data != nil && data["validators"] != nil {
			return c.JSON(data["validators"])
		}
		return c.JSON([]interface{}{})
	})

	// /api/mempool
	app.Get("/api/mempool", func(c *fiber.Ctx) error {
		data, _ := fetchRpc("thunder_getMempool", nil)
		if data != nil && data["mempool"] != nil {
			return c.JSON(data["mempool"])
		}
		return c.JSON([]interface{}{})
	})

	// /api/tx/:hash
	app.Get("/api/tx/:hash", func(c *fiber.Ctx) error {
		hash := c.Params("hash")
		tx, _ := fetchRpc("thunder_getTransactionByHash", map[string]interface{}{"hash": hash})
		if tx != nil {
			return c.JSON(tx)
		}
		return c.Status(404).JSON(fiber.Map{"error": "Transaction not found"})
	})

	// /api/account/:address
	app.Get("/api/account/:address", func(c *fiber.Ctx) error {
		address := c.Params("address")
		var balance int64 = 0

		balData, _ := fetchRpc("thunder_getBalance", map[string]interface{}{"address": address})
		if balData != nil && balData["balance"] != nil {
			// Thunder node returns numbers, but could be parsed as float64 by encoding/json
			switch v := balData["balance"].(type) {
			case float64:
				balance = int64(v)
			case string:
				parsed, _ := strconv.ParseInt(v, 10, 64)
				balance = parsed
			}
		}

		transactions := []interface{}{}
		txData, _ := fetchRpc("thunder_getTransactionsByAddress", map[string]interface{}{"address": address})
		if txData != nil && txData["transactions"] != nil {
			transactions = txData["transactions"].([]interface{})
		}

		if balance == 0 && len(transactions) == 0 {
			return c.Status(404).JSON(fiber.Map{"error": "Wallet not found or never used"})
		}

		return c.JSON(fiber.Map{
			"address":      address,
			"balance":      balance,
			"type":         "Wallet",
			"transactions": transactions,
		})
	})

	fmt.Println("🚀 Golang Fiber API Server listening on port 5050")
	if err := app.Listen(":5050"); err != nil {
		fmt.Printf("Startup error: %v\n", err)
	}
}
