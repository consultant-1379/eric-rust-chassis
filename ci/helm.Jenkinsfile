#!/usr/bin/env groovy

def bob = "./bob/bob -r ruleset2.0.yaml"

try {
    stage("Test deployment") {
        sh "${bob} test-deploy"
    }
} catch (e) {
    throw e
}
