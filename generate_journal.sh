#!/bin/bash

today=`date -u +%Y-%m-%d`

echo "account Assets       ; type:A, things I own
account Liabilities  ; type:L, things I owe
account Equity       ; type:E, net worth or 'total investment'; equal to A - L
account Expenses     ; type:X, outflow categories; part of E, separated for reporting

$today Income
    Assets      3000 USD
    Income

$today Expenses 1
    Expenses:1   12 USD
    Assets

$today Expenses 2
    Expenses:2   30 USD
    Assets

$today Expenses 3
    Expenses:3   350 USD
    Assets

$today Expenses 4
    Expenses:1   4.50 USD
    Assets" > testing.journal