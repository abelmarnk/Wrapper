# Wrapper

---

## Description

This program was written as a **demonstration of a possible solution to rogue packages, rogue bots, and other intrusions** that aim to steal keys or send unauthorized transactions.

### How It Works

1. **User Configuration**  
   The user describes details about the kinds of transactions they frequently make — including:
   - Accounts involved  
   - Typical data and parameters  
   - The program or context for these transactions  

2. **Starter Key & Controlled Account**  
   The user deposits some funds into an **account controlled by this program**.  
   - The program is given a **starter key**, which is only allowed to initiate transactions that match the user’s described patterns.  
   - Users can **withdraw funds at any time**.

3. **Transaction Execution**  
   Instead of using their primary keys for every transaction:
   - The user uses the **starter key**.  
   - If the key is ever compromised, attackers can **only send pre-approved transaction forms**.  
   - In contexts like automated bots or tools, the recipients of those transactions would still be **accounts controlled by the user**, so no real harm is done.

> This design reduces risk by reducing direct exposure of primary keys.

---

## Performance Notes

Due to the **additional compute unit (CU)** usage though little, this program is best suited for **high-priority transactions**, the program is still being **optimized and reviewed**.

---

## Transaction Form Validity

Transaction forms that can be committed may have different validity rules, such as:

- **One-time use**
- **Multiple uses**
- **Time-bound validity** (e.g., valid for a specific duration)

---

## Summary

This wrapper provides a safer interface for executing frequent or automated transactions by **abstracting away direct key usage** and allowing only controlled, pre-approved transaction forms to be sent.

