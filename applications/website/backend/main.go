package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
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
		AllowHeaders: "Origin, Content-Type, Accept",
	}))

	// Auto-verify system contracts on startup
	go func() {
		// Wait a bit for the RPC node to be ready
		time.Sleep(3 * time.Second)
		res, err := fetchRpc("thunder_getSystemInfo", map[string]interface{}{})
		if err == nil && res != nil {
			if resultObj, ok := res["result"].(map[string]interface{}); ok {
				if systemContracts, ok := resultObj["system_contracts"].(map[string]interface{}); ok {
					for name, addrObj := range systemContracts {
						addr, ok := addrObj.(string)
						if !ok {
							continue
						}
						// Dynamically search for the contract source in Core-Engine
						var sourcePath string
						rootDir := "../../../Core-Engine/contracts"
						filepath.Walk(rootDir, func(path string, info os.FileInfo, err error) error {
							if err == nil && !info.IsDir() && info.Name() == name+".ths" {
								sourcePath = path
							}
							return nil
						})

						if sourcePath != "" {
							sourceBytes, err := os.ReadFile(sourcePath)
							if err == nil {
								os.MkdirAll("./data/verified_contracts", 0755)
								destPath := fmt.Sprintf("./data/verified_contracts/%s.ths", addr)
								os.WriteFile(destPath, sourceBytes, 0644)
								fmt.Printf("✅ Automatically verified System Contract: %s at %s\n", name, addr)
							} else {
								fmt.Printf("❌ Failed to read %s: %v\n", name, err)
							}
						} else {
							fmt.Printf("⚠️ Source for System Contract %s not found in %s\n", name, rootDir)
						}
					}
				}
			}
		} else {
			fmt.Printf("❌ Failed to fetch system info: %v\n", err)
		}
	}()

	// Serve the genesis file as an endpoint
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

		accData, _ := fetchRpc("thunder_getAccount", map[string]interface{}{"address": address})
		if accData != nil && accData["balance"] != nil {
			// Thunder node returns numbers, but could be parsed as float64 by encoding/json
			switch v := accData["balance"].(type) {
			case float64:
				balance = int64(v)
			case string:
				parsed, _ := strconv.ParseInt(v, 10, 64)
				balance = parsed
			}
		}

		var isContract bool = false
		var codeLength int64 = 0
		if accData != nil {
			if v, ok := accData["is_contract"].(bool); ok {
				isContract = v
			}
			if cl, ok := accData["code_length"].(float64); ok {
				codeLength = int64(cl)
			}
		}

		transactions := []interface{}{}
		txData, _ := fetchRpc("thunder_getTransactionsByAddress", map[string]interface{}{"address": address})
		if txData != nil && txData["transactions"] != nil {
			transactions = txData["transactions"].([]interface{})
		}

		if balance == 0 && len(transactions) == 0 && !isContract {
			return c.Status(404).JSON(fiber.Map{"error": "Wallet not found or never used"})
		}

		return c.JSON(fiber.Map{
			"address":      address,
			"balance":      balance,
			"isContract":   isContract,
			"codeLength":   codeLength,
			"transactions": transactions,
		})
	})

	// /api/contract/verify
	app.Post("/api/contract/verify", func(c *fiber.Ctx) error {
		type VerifyReq struct {
			Address    string `json:"address"`
			SourceCode string `json:"sourceCode"`
		}
		var req VerifyReq
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request body"})
		}

		res, err := fetchRpc("thunder_verifyContract", map[string]interface{}{
			"address": req.Address,
			"source":  req.SourceCode,
		})
		
		if err != nil || res == nil {
			return c.Status(500).JSON(fiber.Map{"error": "Verification RPC failed"})
		}
		
		if verified, ok := res["verified"].(bool); ok && verified {
			// Save the source code
			filePath := fmt.Sprintf("./data/verified_contracts/%s.ths", req.Address)
			os.MkdirAll("./data/verified_contracts", 0755)
			err = os.WriteFile(filePath, []byte(req.SourceCode), 0644)
			if err != nil {
				return c.Status(500).JSON(fiber.Map{"error": "Failed to save verified source code"})
			}
			return c.JSON(fiber.Map{"verified": true})
		}
		
		return c.Status(400).JSON(fiber.Map{"error": "Source code does not match deployed bytecode", "verified": false})
	})

	// /api/contract/source/:address
	app.Get("/api/contract/source/:address", func(c *fiber.Ctx) error {
		address := c.Params("address")
		filePath := fmt.Sprintf("./data/verified_contracts/%s.ths", address)
		content, err := os.ReadFile(filePath)
		if err != nil {
			return c.Status(404).JSON(fiber.Map{"error": "Source code not found"})
		}
		return c.JSON(fiber.Map{
			"address":    address,
			"sourceCode": string(content),
		})
	})

	// /api/tx/send
	app.Post("/api/tx/send", func(c *fiber.Ctx) error {
		type SendReq struct {
			To         string `json:"to"`
			Amount     string `json:"amount"`
			PrivateKey string `json:"private_key"`
		}
		var req SendReq
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request body"})
		}

		cmd := exec.Command("cargo", "run", "--bin", "thunder-cli", "--", "tx", "send", "--to", req.To, "--amount", req.Amount)
		cmd.Dir = "/Applications/XAMPP/xamppfiles/htdocs/Thunder-Network"
		cmd.Stdin = bytes.NewBufferString(req.PrivateKey + "\n")
		
		output, err := cmd.CombinedOutput()
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to execute tx send", "details": string(output)})
		}

		outStr := string(output)
		if strings.Contains(outStr, "❌") || strings.Contains(outStr, "Error") || strings.Contains(outStr, "error") {
			return c.Status(400).JSON(fiber.Map{"error": outStr})
		}

		return c.JSON(fiber.Map{
			"success": true,
			"output":  outStr,
		})
	})

	// /api/tx/stake
	app.Post("/api/tx/stake", func(c *fiber.Ctx) error {
		type StakeReq struct {
			Amount     string `json:"amount"`
			Duration   int    `json:"duration"`
			PrivateKey string `json:"private_key"`
		}
		var req StakeReq
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request body"})
		}
		
		pkHex := req.PrivateKey
		pkHex = strings.TrimPrefix(pkHex, "0x")

		seed, err := hex.DecodeString(pkHex)
		if err != nil || len(seed) != 32 {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid private key format"})
		}
		
		cmd := exec.Command("cargo", "run", "--bin", "thunder-cli", "--", "tx", "stake", "--amount", req.Amount, "--duration", strconv.Itoa(req.Duration))
		cmd.Dir = "/Applications/XAMPP/xamppfiles/htdocs/Thunder-Network"
		cmd.Stdin = bytes.NewBufferString(pkHex + "\n")
		
		output, err := cmd.CombinedOutput()
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to execute tx stake", "details": string(output)})
		}

		outStr := string(output)
		if strings.Contains(outStr, "❌") || strings.Contains(outStr, "Error") || strings.Contains(outStr, "error") {
			return c.Status(400).JSON(fiber.Map{"error": outStr})
		}

		return c.JSON(fiber.Map{
			"success": true,
			"output":  outStr,
		})
	})

	// /api/tx/unstake
	app.Post("/api/tx/unstake", func(c *fiber.Ctx) error {
		type UnstakeReq struct {
			PrivateKey string `json:"private_key"`
		}
		var req UnstakeReq
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request body"})
		}
		
		pkHex := req.PrivateKey
		pkHex = strings.TrimPrefix(pkHex, "0x")

		seed, err := hex.DecodeString(pkHex)
		if err != nil || len(seed) != 32 {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid private key format"})
		}
		
		cmd := exec.Command("cargo", "run", "--bin", "thunder-cli", "--", "tx", "unstake")
		cmd.Dir = "/Applications/XAMPP/xamppfiles/htdocs/Thunder-Network"
		cmd.Stdin = bytes.NewBufferString(pkHex + "\n")
		
		output, err := cmd.CombinedOutput()
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to execute tx unstake", "details": string(output)})
		}

		outStr := string(output)
		if strings.Contains(outStr, "❌") || strings.Contains(outStr, "Error") || strings.Contains(outStr, "error") {
			return c.Status(400).JSON(fiber.Map{"error": outStr})
		}

		return c.JSON(fiber.Map{
			"success": true,
			"output":  outStr,
		})
	})

	// /api/tx/deploy
	app.Post("/api/tx/deploy", func(c *fiber.Ctx) error {
		type DeployReq struct {
			PrivateKey string `json:"private_key"`
			File       string `json:"file"`
		}
		var req DeployReq
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request body"})
		}
		
		pkHex := req.PrivateKey
		pkHex = strings.TrimPrefix(pkHex, "0x")

		cmd := exec.Command("cargo", "run", "--bin", "thunder-cli", "--", "tx", "deploy", "--file", req.File)
		cmd.Dir = "/Applications/XAMPP/xamppfiles/htdocs/Thunder-Network/Core-Engine"
		cmd.Stdin = bytes.NewBufferString(pkHex + "\n")
		
		output, err := cmd.CombinedOutput()
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to execute tx deploy", "details": string(output)})
		}

		outStr := string(output)
		if strings.Contains(outStr, "❌") || strings.Contains(outStr, "Error") || strings.Contains(outStr, "error") {
			return c.Status(400).JSON(fiber.Map{"error": outStr})
		}

		return c.JSON(fiber.Map{
			"success": true,
			"output":  outStr,
		})
	})

	// /api/tx/call
	app.Post("/api/tx/call", func(c *fiber.Ctx) error {
		type CallReq struct {
			PrivateKey string `json:"private_key"`
			To         string `json:"to"`
			Function   string `json:"function"`
			Amount     string `json:"amount"`
		}
		var req CallReq
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request body"})
		}
		
		pkHex := req.PrivateKey
		pkHex = strings.TrimPrefix(pkHex, "0x")

		amt := req.Amount
		if amt == "" {
			amt = "0"
		}

		cmd := exec.Command("cargo", "run", "--bin", "thunder-cli", "--", "tx", "call", "--to", req.To, "--function", req.Function, "--amount", amt)
		cmd.Dir = "/Applications/XAMPP/xamppfiles/htdocs/Thunder-Network/Core-Engine"
		cmd.Stdin = bytes.NewBufferString(pkHex + "\n")
		
		output, err := cmd.CombinedOutput()
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to execute tx call", "details": string(output)})
		}

		outStr := string(output)
		if strings.Contains(outStr, "❌") || strings.Contains(outStr, "Error") || strings.Contains(outStr, "error") {
			return c.Status(400).JSON(fiber.Map{"error": outStr})
		}

		return c.JSON(fiber.Map{
			"success": true,
			"output":  outStr,
		})
	})

	// /api/faucet
	app.Post("/api/faucet", func(c *fiber.Ctx) error {
		type FaucetReq struct {
			Address string `json:"address"`
		}
		var req FaucetReq
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request body"})
		}

		// Hardcoded faucet rules: 1000 THDR per request (scale to nano-THDR)
		res, err := fetchRpc("thunder_requestFaucet", map[string]interface{}{
			"address":   req.Address,
			"amount":    1000000000000,
			"gas_price": 1,
		})

		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
		return c.JSON(res)
	})

	// /api/wallet/derive-address
	app.Post("/api/wallet/derive-address", func(c *fiber.Ctx) error {
		type DeriveReq struct {
			Seed  string `json:"seed"`
			Index int    `json:"index"`
		}
		var req DeriveReq
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request body"})
		}
		
		cmd := exec.Command("cargo", "run", "--bin", "thunder-cli", "--", "wallet", "derive-address", "--seed", req.Seed, "--index", strconv.Itoa(req.Index))
		cmd.Dir = "/Applications/XAMPP/xamppfiles/htdocs/Thunder-Network"
		
		output, err := cmd.CombinedOutput()
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to derive address", "details": string(output)})
		}
		
		outStr := strings.TrimSpace(string(output))
		lines := strings.Split(outStr, "\n")
		var jsonLine string
		for _, line := range lines {
			if strings.HasPrefix(strings.TrimSpace(line), "{") {
				jsonLine = line
				break
			}
		}
		
		var result map[string]interface{}
		if err := json.Unmarshal([]byte(jsonLine), &result); err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to parse derive output", "details": outStr})
		}
		
		return c.JSON(result)
	})

	// /api/wallet/generate-seed
	app.Post("/api/wallet/generate-seed", func(c *fiber.Ctx) error {
		cmd := exec.Command("cargo", "run", "--bin", "thunder-cli", "--", "wallet", "generate-seed")
		cmd.Dir = "/Applications/XAMPP/xamppfiles/htdocs/Thunder-Network"
		
		output, err := cmd.CombinedOutput()
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to generate seed", "details": string(output)})
		}
		
		outStr := strings.TrimSpace(string(output))
		// find the json output
		lines := strings.Split(outStr, "\n")
		var jsonLine string
		for _, line := range lines {
			if strings.HasPrefix(strings.TrimSpace(line), "{") {
				jsonLine = line
				break
			}
		}
		
		var result map[string]interface{}
		if err := json.Unmarshal([]byte(jsonLine), &result); err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Failed to parse seed output", "details": outStr})
		}
		
		return c.JSON(result)
	})

	fmt.Println("🚀 Golang Fiber API Server listening on port 5050")
	if err := app.Listen(":5050"); err != nil {
		fmt.Printf("Startup error: %v\n", err)
	}
}
