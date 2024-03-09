export default function LinksContainer({children}) {
    // Component taken from
    // https://www.mambaui.com/components/gallery
    return (
        <>
            <section className="py-6 dark:bg-gray-800">
                <div className="container flex flex-col justify-center p-4 mx-auto">
                    <div className="grid grid-cols-1 gap-4 lg:grid-cols-4 sm:grid-cols-2">
                        {children}
                    </div>
                </div>
            </section>
        </>
    )
}
