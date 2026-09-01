import { ethers } from 'ethers'

let wallet: ethers.Wallet | null = null;

// Initialize wallet from storage if it exists
chrome.storage.local.get(['thunder_private_key'], (result) => {
  if (result.thunder_private_key) {
    try {
      wallet = new ethers.Wallet(result.thunder_private_key as string)
      console.log('Thunder Wallet loaded in background. Address:', wallet.address)
    } catch (error) {
      console.error('Failed to load wallet', error)
    }
  }
})

// Listen for storage changes to update wallet if user creates/imports one
chrome.storage.onChanged.addListener((changes, namespace) => {
  if (namespace === 'local' && changes.thunder_private_key) {
    try {
      wallet = new ethers.Wallet(changes.thunder_private_key.newValue as string)
      console.log('Thunder Wallet updated in background. Address:', wallet.address)
    } catch (error) {
      console.error('Failed to update wallet', error)
    }
  }
})

// Listen to connections from popup and content scripts
chrome.runtime.onMessage.addListener((request: any, _sender: any, sendResponse: any) => {
  if (request.method === 'eth_accounts' || request.method === 'thunder_accounts') {
    if (wallet) {
      sendResponse([wallet.address])
    } else {
      sendResponse([])
    }
    return true // Indicates async response
  }

  if (request.method === 'eth_sendTransaction' || request.method === 'thunder_sendTransaction') {
    console.log('Received transaction request:', request.params)
    // In a real wallet, this would trigger a popup window for the user to confirm.
    // For MVP, we will mock a transaction hash response
    const mockTxHash = '0x' + Array.from({length: 64}, () => Math.floor(Math.random()*16).toString(16)).join('')
    sendResponse(mockTxHash)
    return true
  }

  // Handle setting network
  if (request.method === 'wallet_switchEthereumChain') {
    console.log('Switching network to:', request.params[0].chainId)
    sendResponse(null)
    return true
  }

  return false
})
