import { deepStrictEqual, ok } from "node:assert"
import { type IssueFeaturesRecord, IssueFeatureStore } from "./mod.ts"

Deno.test("IssueFeatureStore#preload", () => {
    const store = new IssueFeatureStore()

    const record1: IssueFeaturesRecord = {
        issueId: "1",
        features: {
            operation: "Turn on the switch",
            expectedBehavior: "The device is turned on",
            actualBehavior: "The device is not turned on",
        },
    }
    const record2: IssueFeaturesRecord = {
        issueId: "2",
        features: {
            operation: "Turn off the switch",
            phenomenon: "The device remains turned on instead if being turned on",
        },
    }
    const records: IssueFeaturesRecord[] = [
        record1,
        record2,
        { // This one is invalid will be ignored
            issueId: "3",
            features: {},
        },
    ]

    store.preload(records)

    deepStrictEqual(store.getRecord("1"), record1)
    deepStrictEqual(store.getRecord("2"), record2)
    deepStrictEqual(store.getRecord("3"), null)
    deepStrictEqual(store.getRecord("4"), null)
})

Deno.test("IssueFeatureStore#addRecord", () => {
    const store = new IssueFeatureStore()

    const record: IssueFeaturesRecord = {
        issueId: "1",
        features: {
            operation: "Turn on the switch",
            expectedBehavior: "The device is turned on",
            actualBehavior: "The device is not turned on",
        },
    }

    store.addRecord(record)
    deepStrictEqual(store.getRecord("1"), record)
    deepStrictEqual(store.getRecord("2"), null)
})

Deno.test("IssueFeatureStore#removeRecord", () => {
    const store = new IssueFeatureStore()

    const record: IssueFeaturesRecord = {
        issueId: "1",
        features: {
            operation: "Turn on the switch",
            expectedBehavior: "The device is turned on",
            actualBehavior: "The device is not turned on",
        },
    }

    store.addRecord(record)
    deepStrictEqual(store.getRecord("1"), record)

    const removed = store.removeRecord("1")
    deepStrictEqual(removed, true)
    deepStrictEqual(store.getRecord("1"), null)

    // Removing a non-existing record should return false
    const removedAgain = store.removeRecord("1")
    deepStrictEqual(removedAgain, false)
})

Deno.test("IssueFeatureStore#findSimilarRecords", async () => {
    const store = new IssueFeatureStore()

    const record1: IssueFeaturesRecord = {
        issueId: "1",
        features: {
            operation: "Turn on the switch",
            expectedBehavior: "The device is turned on",
            actualBehavior: "The device is not turned on",
        },
    }
    const record2: IssueFeaturesRecord = {
        issueId: "2",
        features: {
            operation: "Turn off the switch",
            phenomenon: "The device remains turned on instead if being turned on",
        },
    }

    store.preload([record1, record2])

    const signal = AbortSignal.timeout(5_000)
    const similarRecords = await store.findSimilarRecords({
        operation: "Turn on the switch",
        expectedBehavior: "The device turns on",
        actualBehavior: "The device does not turn on",
    }, { topN: 2, signal })

    deepStrictEqual(similarRecords.length, 1)
    deepStrictEqual(similarRecords[0].issueId, "1")
    deepStrictEqual(similarRecords[0].features, record1.features)
    ok(similarRecords[0].score > 0.8)
})
