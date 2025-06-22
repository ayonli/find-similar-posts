import { deepStrictEqual, ok } from "node:assert"
import func from "@ayonli/jsext/func"
import { type IssueFeaturesRecord, IssueFeatureStore } from "./mod.ts"

Deno.test("IssueFeatureStore#constructor", () => {
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

    const store = new IssueFeatureStore(records)

    deepStrictEqual(store.getRecord("1"), record1)
    deepStrictEqual(store.getRecord("2"), record2)
    deepStrictEqual(store.getRecord("3"), null)
    deepStrictEqual(store.getRecord("4"), null)
})

Deno.test("IssueFeatureStore#setRecord", () => {
    const store = new IssueFeatureStore()

    const record: IssueFeaturesRecord = {
        issueId: "1",
        features: {
            operation: "Turn on the switch",
            expectedBehavior: "The device is turned on",
            actualBehavior: "The device is not turned on",
        },
    }

    store.setRecord(record)
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

    store.setRecord(record)
    deepStrictEqual(store.getRecord("1"), record)

    const removed = store.removeRecord("1")
    deepStrictEqual(removed, true)
    deepStrictEqual(store.getRecord("1"), null)

    // Removing a non-existing record should return false
    const removedAgain = store.removeRecord("1")
    deepStrictEqual(removedAgain, false)
})

Deno.test("IssueFeatureStore#findSimilarRecords", async () => {
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

    const store = new IssueFeatureStore([record1, record2])

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

Deno.test("IssueFeatureStore.fromDB_mysql", async () => {
    const store = await IssueFeatureStore.fromDB({
        url: Deno.env.get("MYSQL_URL") ?? "",
        table: "issue_features",
    })

    deepStrictEqual(
        store.getRecord("1"),
        {
            issueId: "1",
            features: {
                operation: "Turn on the switch",
                expectedBehavior: "The device is turned on",
                actualBehavior: "The device is not turned on",
            },
        } satisfies IssueFeaturesRecord,
    )
    deepStrictEqual(
        store.getRecord("2"),
        {
            issueId: "2",
            features: {
                operation: "Turn off the switch",
                phenomenon: "The device remains turned on instead if being turned on",
            },
        } satisfies IssueFeaturesRecord,
    )
    deepStrictEqual(store.getRecord("3"), null)
})

Deno.test("IssueFeatureStore.fromDB_postgres", async () => {
    const store = await IssueFeatureStore.fromDB({
        url: Deno.env.get("PG_URL") ?? "",
        table: "issue_features",
    })

    deepStrictEqual(
        store.getRecord("1"),
        {
            issueId: "1",
            features: {
                operation: "Turn on the switch",
                expectedBehavior: "The device is turned on",
                actualBehavior: "The device is not turned on",
            },
        } satisfies IssueFeaturesRecord,
    )
    deepStrictEqual(
        store.getRecord("2"),
        {
            issueId: "2",
            features: {
                operation: "Turn off the switch",
                phenomenon: "The device remains turned on instead if being turned on",
            },
        } satisfies IssueFeaturesRecord,
    )
    deepStrictEqual(store.getRecord("3"), null)
})

Deno.test("IssueFeatureStore.fromDB_sqlite", async () => {
    const store = await IssueFeatureStore.fromDB({
        url: "sqlite:./assets/issue_mgr.db",
        table: "issue_features",
    })

    deepStrictEqual(
        store.getRecord("1"),
        {
            issueId: "1",
            features: {
                operation: "Turn on the switch",
                expectedBehavior: "The device is turned on",
                actualBehavior: "The device is not turned on",
            },
        } satisfies IssueFeaturesRecord,
    )
    deepStrictEqual(
        store.getRecord("2"),
        {
            issueId: "2",
            features: {
                operation: "Turn off the switch",
                phenomenon: "The device remains turned on instead if being turned on",
            },
        } satisfies IssueFeaturesRecord,
    )
    deepStrictEqual(store.getRecord("3"), null)
})

Deno.test("IssueFeatureStore.fromCSV", async () => {
    const store = await IssueFeatureStore.loadCSV("./assets/issue_features.csv")

    deepStrictEqual(
        store.getRecord("1"),
        {
            issueId: "1",
            features: {
                operation: "Turn on the switch",
                expectedBehavior: "The device is turned on",
                actualBehavior: "The device is not turned on",
            },
        } satisfies IssueFeaturesRecord,
    )
    deepStrictEqual(
        store.getRecord("2"),
        {
            issueId: "2",
            features: {
                operation: "Turn off the switch",
                phenomenon: "The device remains turned on instead if being turned on",
            },
        } satisfies IssueFeaturesRecord,
    )
    deepStrictEqual(store.getRecord("3"), null)
})

Deno.test(
    "IssueFeatureStore.toCSV",
    func(async (defer) => {
        const input = "./assets/issue_features.csv"
        const output = "./assets/issue_features_copy.csv"
        const store1 = await IssueFeatureStore.loadCSV(input)
        await store1.dumpCSV(output)
        defer(() => Deno.remove(output))

        const store2 = await IssueFeatureStore.loadCSV(output)
        deepStrictEqual(store2.getRecord("1"), store1.getRecord("1"))
        deepStrictEqual(store2.getRecord("2"), store1.getRecord("2"))
        deepStrictEqual(store2.getRecord("3"), null)
    }),
)
