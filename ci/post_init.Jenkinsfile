#!/usr/bin/env groovy

def bob = "./bob/bob -r ruleset2.0.yaml"

try {
    stage("Build Image") {
        sh "${bob} create-build-image"
    }
    stage("Build") {
        sh "${bob} build"
    }
    stage("Doc") {
        sh "${bob} doc"
        archiveArtifacts allowEmptyArchive: false, artifacts: "target/doc/**"
    }
    stage("Test") {
        sh "${bob} test"
    }
} catch (e) {
    throw e
}
