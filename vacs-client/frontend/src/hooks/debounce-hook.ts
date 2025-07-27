import {useCallback, useRef, useState} from "preact/hooks";

export function useAsyncDebounce<TArgs extends unknown[], TResult>(
    fn: (...args: TArgs) => Promise<TResult>
): (...args: TArgs) => Promise<TResult | void> {
    const loading = useRef<boolean>(false);

    return useCallback(async (...args: TArgs): Promise<TResult | void> => {
       if (loading.current) return;
       loading.current = true;
       try {
           return await fn(...args);
       } finally {
           loading.current = false;
       }
    }, [fn]);
}

export function useAsyncDebounceState<TArgs extends unknown[], TResult>(
    fn: (...args: TArgs) => Promise<TResult>
): [(...args: TArgs) => Promise<TResult | void>, boolean] {
    const [loading, setLoading] = useState<boolean>(false);

    const wrapped = useCallback(async (...args: TArgs): Promise<TResult | void> => {
        if (loading) return;
        setLoading(true);
        try {
            return await fn(...args);
        } finally {
            setLoading(false);
        }
    }, [fn]);

    return [wrapped, loading];
}